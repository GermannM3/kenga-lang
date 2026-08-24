# GENESIS V0 — цикл самороста Kenga Prophet на верифицированном опыте

Статус: **СПЕЦИФИКАЦИЯ, ЗАБЛОКИРОВАНА Гейтом входа** (см. §5).
Эксперимент «модель-ребёнок растёт на собственном верифицированном опыте».
Ничего из описанного здесь пока не запускалось; веса в `minds/` не трогаются.

Инфраструктура, на которую опираемся (всё уже есть):

| Компонент | Что даёт |
|---|---|
| `tools/kenchat.py` | `gen_tokens`, `gen_verified` (self-consistency), `make_valid_program`, `run_via_kenga_lite` → `(rc, stdout, stderr)` — полный generate→compile→run→value конвейер |
| `tools/train_m3.py` | numpy-трансформер, AdamOpt, per-position causal LM, сохранение в текст `[name] shape=[...] values` (scale=1000) |
| `tools/corpus_eval.py` | factory-test метрики по категориям (`--category bind`) |
| `tools/realgen_eval.py` | генерация на реальном held-out коде: compile/match/pass@k |
| `minds/corpus_factory/split_v2/{train,test}.jsonl` | verified-программы с полями `src/out/category/mutants` (arith, bind, chain, loop, rec) |
| `tools/build_repair_corpus.py`, `repair_eval.py` | формат repair-документа `broken\nfixed`, режимы мутантов `run`/`value` |

Родительская модель для v0: `mid_prophet_m52_w.txt` (838 016 параметров,
K=128 D=128 H=8 L=6, V=128, codec `kenga_full.pkl`). Дети живут **только**
в `minds/genesis/` — корень `minds/` остаётся нетронутым.

---

## 1. Цикл Genesis Loop

```
        ┌──────────────────────────────────────────────────────────┐
        │                                                          │
        ▼                                                          │
  base model ──► prompt ──► generation ──► verifier                │
  (parent)      (factory-   (gen_verified)   │                    │
                 префиксы)                  ├─ pass ──► experience buffer
                                            │              │
                                            └─ fail ──► repair pair
                                                           (broken+fixed)
                                                                          │
        child checkpoint ◄── periodic fine-tune ◄── buffer + replay ◄────┘
             │                   (каждые N принятых)
             └──────── ребёнок = новый parent следующего цикла ──────────┘
```

Определения:

* **prompt** — префикс `first_fn_block(rec.src)` случайной записи
  `split_v2/train.jsonl` (не test!). Категория и ожидаемый stdout (`rec.out`)
  известны заранее → verifier может проверять **value**, а не только compile.
  Подмешиваем статические пробы `kenchat.PROBES` как regression-якорь.
* **verifier** — трёхуровневый:
  1. `compile`: программа прошла `kenga-lite run`;
  2. `run`: завершилась с rc==0;
  3. `value`: первая строка stdout == ожидаемой.

  `passed := rc == 0 and (expected is None or first_stdout == expected)`.
* **repair pair** — только когда известен эталон (factory-prompt):
  документ `broken.rstrip() + '\n' + fixed`, где `fixed = rec['src']`
  (тот же формат, что в `build_repair_corpus.py`). Для свободных промптов без
  эталона fail попадает в буфер как «сломанный опыт» (ветка B), но repair-пару
  не создаём — чинить нечем.

### Псевдокод главного цикла

```python
import kenchat, genesis_loop as gl          # gl: buffer/finetune/eval

def genesis_loop(base_weights, codec_path, cfg):
    codec    = kenchat.load_codec_vocab(codec_path)
    weights  = base_weights                       # parent текущего цикла
    buf      = gl.Buffer.load(cfg.buffer_path)
    m_parent = gl.evaluate(weights, codec, tag='parent')

    for cyc in range(1, cfg.cycles + 1):
        prompts = gl.sample_prompts(cfg.prompts_per_cycle)  # factory-train+PROBES
        for prompt, expected, cat, ref_src in prompts:
            toks, src, prog, rc, out, err = kenchat.gen_verified(
                prompt, weights, codec,
                n_samples=cfg.n_samples, max_tokens=cfg.max_tokens,
                temperature=1.0, want=expected)
            first  = out.strip().split('\n')[0] if out else ''
            passed = (rc == 0) and (expected is None or first == expected)
            buf.add(dict(ts=gl.now(), cycle=cyc, cat=cat,
                         prompt=prompt, program=prog, rc=rc,
                         stdout=first, expected=expected, passed=passed))
            if not passed and ref_src is not None:       # fail -> repair pair
                buf.add_repair(broken=prog, fixed=ref_src,
                               mode=gl.fail_mode(rc))    # 'run' | 'value'

        if buf.accepted_since_ft() >= cfg.ft_every:      # N=200 принятых
            child = f'minds/genesis/child_{cyc:03d}_w.txt'
            gl.finetune(parent_txt=weights, buf=buf, branch=cfg.branch,
                        lr=cfg.lr, steps=cfg.ft_steps, out_txt=child)
            m_child = gl.evaluate(child, codec, tag=f'child_{cyc}')
            if not gl.ge_floor(m_child, m_parent, floor=0.95):
                os.remove(child)                         # anti-forgetting gate
                cfg.lr /= 2                              # откат + щадящий LR
                continue                                 # ребёнок НЕ promoted
            weights = child                              # ребёнок -> новый parent
            m_parent = m_child
        gl.journal(cyc, buf.stats(), weights)

    return weights
```

Ключевые свойства: единственный источник истины о корректности — kenga-lite;
ребёнок становится родителем только после анти-забывательного гейта; все
сэмплы (включая проваленные) пишутся в буфер — ветки B/C/D различаются лишь
тем, *что из буфера* идёт в обучение (§2).

---

## 2. Контрольные группы (4 ветки, одинаковый бюджет шагов)

Один бюджет = одно и то же число циклов, одинаковые
`prompts_per_cycle / n_samples / ft_steps / lr`, одинаковый seed-набор
промптов. Отличается только содержимое обучающего потока:

| Ветка | Обучающий поток за цикл | Что измеряем |
|---|---|---|
| **A. Base** | ничего не обучаем | контроль дрейфа пайплайна; greedy-метрики детерминированы → обязаны быть плоскими. Любое движение = баг измерений |
| **B. Unverified** | весь буфер как есть: passed + failed программы (без repair-конкатенаций) | эффект обучения на сыром само-выводе (ожидаем деградацию — негативный контроль по collapse) |
| **C. Verified-only** | только записи `passed == True` | чистый verifier-гейт без repair |
| **D. Verified+repair** | passed-записи + repair-документы (`broken\nfixed`) | полная гипотеза Genesis: успехи + опыт исправления |

Метрики после каждого цикла (обязательный снапшот в
`minds/genesis/evals/<tag>.log`):

1. **factory-test NT accuracy** — next-token accuracy на объединённом потоке
   `split_v2/test.jsonl` (процедура held-out из `train_m3.py`,
   non-overlapping окна K=128);
2. **bind-category**: `python tools/corpus_eval.py --model <tag> --test
   minds/corpus_factory/split_v2/test.jsonl --category bind --limit 144`
   → compile / match(greedy);
3. **realgen**: `python tools/realgen_eval.py --model <tag>`
   → compile / greedy-match по всем runnable held-out файлам.

Сравнительная таблица веток (дельты к parent после каждого цикла):

```
branch cyc NT_acc   bind_compile bind_match realgen_compile realgen_match verdict
A      3   71.86%   5.6%         1.4%       0/6             0/6           flat(ok)
B      3   ...      ...          ...        ...             ...           ?
C      3   ...      ...          ...        ...             ...           ?
D      3   ...      ...          ...        ...             ...           ?
```

(числа A в строке cyc=0 — фактический бейзлайн M5.2).

Успех v0: **D ≥ C > parent ≥ 0.95×parent** по всем метрикам при равном
бюджете; B не хуже A по NT accuracy (иначе подтверждён риск collapse).

---

## 3. Experience buffer

Файл: `minds/genesis/buffer.jsonl` (append-only журнал; активное подмножество
после компактификации — `buffer_active.jsonl`).

Схема записи (обязательные поля):

```json
{"ts": "2026-08-23T12:00:00Z", "cycle": 2,
 "prompt": "fn add(a: i64, b: i64) -> i64 {\n",
 "program": "<полный .kenga исходник, который запускали>",
 "rc": 0, "stdout": "7",
 "expected": "7",
 "passed": true}
```

Расширение (опционально, но настоятельно рекомендуется):

```json
 "h": "3f9a1c...",             // sha1(program)[:16] — ключ дедупликации
 "cat": "bind",                // категория промпта (для балансировки)
 "kind": "attempt" | "repair", // attempt = попытка; repair = пара broken/fixed
 "seed": 3                     // seed сэмплинга gen_tokens (воспроизводимость)
```

Формула D8 (проверяемо внешним читателем):
L(a,b) = mean_t [ cos(log1p(S_a^t), log1p(S_b^t)) * (1 - mean_theta_t/90°) ],
где S^t — топ-32 сингулярных значений, theta_t — главные углы между
топ-32 левыми сингулярными подпространствами U общего-формы тензора t.

Правила:
* ветка A, установка: если L ребёнка к родителю в первом же цикле < 0.95,
  это не «гейт не прошёл» — это ПЕРВОЕ В МИРЕ измерение того, сохраняет ли
  verifier-gated саморост идентичность агента. Оба исхода публикуемы.
Правила:

* **Дедупликация.** `h = sha1(program)[:16]`. Повтор с тем же `h` внутри одного
  цикла не добавляется вовсе; между циклами — добавляется со счётчиком
  `n_seen`, но в обучающий поток программа входит **один раз** (частотное
  подавление повторов — первая защита от self-reinforcement).
* **Лимиты буфера.** `MAX_ATTEMPTS = 5000` записей + `MAX_REPAIRS = 2000`
  repair-пар. При переполнении — FIFO-eviction, причём выталкиваем сначала из
  самой многочисленной категории (гарантия присутствия всех). Журнал не режется
  никогда; усечённый «активный» вид живёт отдельно.
* **Балансировка категорий.** Квоты обучающей выборки пропорциональны
  split_v2-train (~47% arith, 11% bind, 17% chain, 14% loop, 3% rec,
  остаток — PROBES/misc), потолок: ни одна категория > 50% потока. Недобранная
  категория добирается replay'ем из factory-train, а не дубликатами.
* **Ротация промптов**: промпт повторяется не чаще раза в 3 цикла (кольцевой
  список), чтобы рост не сваливался в зубрёжку фиксированного набора задач.

---

## 4. Fine-tune протокол (без забывания)

Реализация — импорт машинерии `train_m3.M3 / backward / AdamOpt`; веса
восстанавливаются из txt через `kenchat.load_tensors` в свежий объект `M3`
(имена тензоров совпадают с `params_map()`: `E_tok, E_pos, {i}:Wq…, Wout,
bout`). `bout` берётся из файла родителя и НЕ переинициализируется униграммой —
это сохраняет калибровку выходного слоя.

Поток обучения за цикл:

```
tokens = tokenize(buffer_stream(branch))       # ~80% потока (по ветке B/C/D)
       + tokenize(replay_factory_train(0.10))  # 10% случайных записей split_v2/train.jsonl
       + tokenize(PROBES_seeds)                # 10% якорных сидов-программ
```

Гиперпараметры дообучения:

| Параметр | Значение | Обоснование |
|---|---|---|
| LR | **0.0005** (= базовый M3_LR 0.005 / 10) | малый шаг: сдвиг распределения, а не перестройка |
| FT_STEPS | 400 за цикл | сопоставимо по времени с одним прогоном eval |
| BATCH | 32 | CPU-бюджет для ~838K параметров |
| grad clip | max_norm = 1.0 | защита от выброса градиента на repair-документах |
| Периодичность | каждые **N = 200** принятых (`passed==True`) образцов | раньше — шум, позже — дрейф между апдейтами |

Чекпоинты: `minds/genesis/child_{NNN}_w.txt` в том же текстовом формате
(scale=1000), рядом `child_{NNN}_meta.txt`:

```
parent=m52                      # или child_002 при каскаде
branch=D
cycle=3                         # номер цикла Genesis
buffer_size=1240                # активных attempts на момент FT
repairs=310
ft_steps=400  lr=0.0005  batch=32
replay_frac=0.10
nt_acc=72.10  bind_compile=8/144  realgen_compile=1/6   # снапшот метрик
ts=2026-08-23T12:34:56Z
```

Rollback-правило: если анти-забывательный гейт (§6) не пройден — файл ребёнка
удаляется, LR делится пополам, цикл перезапускается от прежнего родителя.
Буфер при этом сохраняется: выбрасываем плохие веса, но не опыт.

---

## 5. Гейт входа в Genesis

Genesis v0 **не стартует**, пока родитель не проходит порог на реальном коде:

```
python tools/realgen_eval.py --model m52
```

Условия старта (все обязательны):

* **Binding gate**: compile ≥ **40%** и match(pass@k) ≥ **20%** на
  bind-категории template-split (`corpus_eval.py --category bind`) —
  модель умеет связывать вызов с определением без name-shortcut'ов;
* **Real-gen gate**: compile-rate ≥ **30%** по всем runnable held-out
  файлам; semantic match (greedy) ≥ **10%**;
* **Anti-forgetting baseline**: у parent зафиксированы factory-test NT,
  bind-метрики и realgen-метрики (сравнение §6 после каждого цикла).

Факт на сегодня: **M5.2 даёт bind compile 5.6% / real-gen 0/6** → оба гейта
не пройдены, Genesis v0 отложен до M5.3+/M5.4. Пока гейт закрыт, разрешены
только подготовительные шаги §7:
`/learn`, `/buffer` в ручном режиме — они копят проверенный опыт и не обучают
модель автоматически.

### Identity gate (Z x Kenga, ZK-2)

Каждый цикл Genesis измеряет спектральную близость ребёнка к родителю
(`tools/zlineage.py::pair_drift`, паспорт = top-32 log-спектры + главные
углы подпространств, агрегат L).

Измерено на родовой линии Kenga (Z_LINEAGE.md):
* тот же прогон (снапшот -> финал): L = 1.0000
* разные прогоны одного семейства: L = 0.30..0.49
* случайный ноль: L = 0.284

Правила:
* ветка A (Base control): только ЛОГИРОВАНИЕ L за цикл, без блокировки;
* ветки C/D (Verified / Verified+repair): БЛОКИРУЮЩИЙ гейт —
  `L(child, parent) >= 0.95` ИЛИ точное совпадение маркера;
  иначе ребёнок помечается `different_agent` и в родовую линию не идёт.
* порог 0.95 — кандидат D8 (подтверждён тем, что все замеренные пары
  «разные прогоны» лежат ниже 0.50 при нулевом уровне 0.28).

Технически гейт оформляется функцией в `genesis_loop.py` и вызывается первым
действием `main()`; при отказе печатается причина и код выхода 2:

```python
def entry_gate(model_tag, min_bind_compile=0.40, min_bind_match=0.20,
               min_compile=0.30, min_match=0.10):
    bind = parse_bind_log(run_bind_eval(model_tag))
    rg = parse_realgen_log(run_realgen_eval(model_tag))
    return (bind.compile_rate >= min_bind_compile
            and bind.match_rate >= min_bind_match
            and rg.compile_rate >= min_compile
            and rg.match_rate >= min_match)
```

Сухая проверка механики возможна в режиме `--dry-run`
(генерация+верификация+буфер без единого шага обучения) — гейтом не
блокируется.

---

## 6. Риски и митигации

| Риск | Симптом | Митигация |
|---|---|---|
| **Model collapse** (обучение на собственных выводах) | падение разнообразия программ, рост доли дубликатов `h`, NT accuracy ползёт вниз при неизменном pass-rate на старых промптах | (1) verifier-гейт: в C/D градиент получают только программы, реально скомпилировавшиеся и напечатавшие верное значение; (2) replay 10% внешнего factory-train каждый цикл — внешний поток не зависит от модели; (3) при необходимости докрутка нового verified-корпуса через `corpus_factory.py` (внешний учитель); (4) мониторинг `unique(h)/records` за цикл, падение ниже 0.5 — стоп-условие; ветка B — намеренный негативный контроль |
| **Катастрофическое забывание** | NT/bind/realgen просели против parent | жёсткое правило: после каждого цикла все три группы метрик ≥ **0.95× parent**, иначе rollback + LR/2 (§4); плюс низкий LR, grad clip, replay; bout не переинициализируется |
| **Бесконечный рост буфера** | время FT и память растут линейно с возрастом эксперимента | лимиты §3 (5000 attempts / 2000 repairs), дедупликация, FIFO-eviction с защитой категорий; обучающий поток цикла ограничен сверху константой токенов — стоимость цикла постоянна |
| Висящие программы | `kenga-lite` подвисает на self-generated коде | `timeout=10` в `run_via_kenga_lite`; rc=-1 → fail-запись; таймаут — валидный опыт ветки B, repair-пару не создаёт |
| Нечестное сравнение веток | у D формально больше токенов за счёт repair-документов | бюджет выравнивается по шагам (STEPS равны), поток ограничен одной квотой токенов; лишнее отсекается равномерно по категориям |

---

## 7. План реализации

### Шаг 1 — минимальные правки `tools/kenchat_cli.py` (~60 строк)

1. Аргументы CLI: `--genesis-dir` (default `minds/genesis`), `--buffer PATH`.
2. Состояние интерактивного режима: путь буфера; при старте автозагрузка
   существующего `buffer.jsonl`, если есть.
3. Новые команды в `interactive()`:
   * `/learn [n] <prompt>` — как `verify n`, но результат (включая fail)
     дописывается в буфер; печатает `PASS/FAIL`, hash, размер буфера; если
     промпт найден в индексе factory-train (по `first_fn_block`) — при fail
     дописывает repair-пару;
   * `/buffer` — статистика: всего/passed%, по категориям, unique-ratio,
     repairs, размер файла;
   * `/grow [cycles]` — вызывает `genesis_loop.grow_cycle(...)` от текущей
     модели, печатает таблицу метрик ребёнок vs parent; работает только при
     пройденном `entry_gate()` (иначе печатает отказ).

### Шаг 2 — новый файл `tools/genesis_loop.py` (каркас, ~150 строк)

```python
"""tools/genesis_loop.py — Genesis Loop v0 (spec: docs/GENESIS_V0.md).
Entry-gated experiment; nothing trains until realgen gates pass."""
import argparse, hashlib, json, os, sys, time
import numpy as np
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kenchat
import train_m3

GENESIS_DIR = 'minds/genesis'
FT_EVERY   = 200      # accepted samples per fine-tune
FT_STEPS   = 400
FT_LR      = 0.0005
REPLAY     = 0.10
FLOOR      = 0.95     # anti-forgetting: child metric >= FLOOR * parent
MAX_ATTEMPTS, MAX_REPAIRS = 5000, 2000



class Buffer:
    """JSONL store: dedup by sha1(program), category quotas, FIFO cap."""
    def __init__(self, path): ...          # load journal if exists
    def add(self, rec): ...                # dedup -> append journal+active
    def add_repair(self, broken, fixed, mode): ...
    def accepted_since_ft(self): ...       # count of new passed records
    def stream(self, branch): ...          # 'B'|'C'|'D' -> training text lines
    def stats(self): ...


def sample_prompts(k):
    """first_fn_block prefixes from split_v2/train.jsonl (+ kenchat.PROBES).
    Returns [(prompt, expected, cat, ref_src_or_None)]."""
    ...


def entry_gate(tag, min_compile=0.30, min_match=0.10): ...
def run_realgen_eval(tag): ...              # subprocess realgen_eval.py
def parse_realgen_log(text): ...


def load_model(txt_path):
    """Rebuild train_m3.M3 from a saved weights file (bout NOT re-inited)."""
    info, t = kenchat.load_tensors(txt_path)
    m = train_m3.M3(info['vocab'], info['k'], info['d'], info['h'],
                    info['layers'], np.random.RandomState(11))
    m.E_tok, m.E_pos = t['E_tok'], t['E_pos']
    m.Wout, m.bout = t['Wout'], t['bout']
    for li in range(info['layers']):
        for nm in ('Wq', 'Wk', 'Wv', 'Wo', 'W1', 'b1', 'W2', 'b2'):
            setattr(m.blocks[li], nm, t[f'{li}:{nm}'])
    return m


def finetune(parent_txt, buf, branch, out_txt, lr=FT_LR, steps=FT_STEPS):
    """Buffer(+repair)+replay stream -> Adam fine-tune -> save child txt
    (same text format, scale=1000) + child meta file."""
    ...


def evaluate(txt_path, codec, tag):
    """NT-acc on split_v2/test stream + corpus_eval(bind subset) + realgen.
    Returns dict; writes minds/genesis/evals/<tag>.log."""
    ...


def ge_floor(m_child, m_parent, floor=FLOOR):
    """All three metric groups >= floor * parent."""
    return all(m_child[k] >= floor * m_parent[k]
               for k in ('nt_acc', 'bind_compile', 'realgen_compile'))


def grow_cycle(parent_txt, codec_path, branch, cfg):
    """One iteration: sample -> gen_verified -> verify -> buffer ->
    maybe finetune -> gate -> promote or rollback. Returns child|None."""
    ...


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--base', required=True)            # e.g. m53
    ap.add_argument('--branch', default='D', choices=list('ABCD'))
    ap.add_argument('--cycles', type=int, default=1)
    ap.add_argument('--dry-run', action='store_true')   # no training at all
    args = ap.parse_args()
    if not args.dry_run and not entry_gate(args.base):
        print('entry gate FAILED: Genesis postponed '
              '(docs/GENESIS_V0.md section 5)')
        return 2
    ...  # genesis_loop(...) from section 1 pseudocode
    return 0


if __name__ == '__main__':
    sys.exit(main())
```

### Шаг 3 — порядок работ

1. `Buffer` + команды `/learn` `/buffer` (ручной сбор опыта, dry-run) — можно
   делать уже сейчас; обучения нет, гейт не задействован.
2. `genesis_loop.py`: `load_model` + `evaluate` (переиспользование held-out
   логики из `train_m3.main`; вызовы `corpus_eval`/`realgen_eval` сабпроцессом)
   + `entry_gate`.
3. `finetune` + `grow_cycle` + `/grow` — активируются автоматически, как только
   модель M5.3+/M5.4 пройдёт гейт §5.
4. Прогон 4 веток × одинаковый бюджет (например 5 циклов × 200 принятых),
   сводная таблица §2 → вывод о жизнеспособности Genesis-подхода.

### Артефакты эксперимента (всё вне корня `minds/`)

```
minds/genesis/
  buffer.jsonl           # append-only журнал опыта
  buffer_active.jsonl    # активное подмножество после лимитов/дедупа
  child_001_w.txt        # чекпоинты детей (формат train_m3)
  child_001_meta.txt
  journal.jsonl          # решения циклов: promote / rollback / gate-fail
  evals/<tag>.log        # снапшоты метрик
```
