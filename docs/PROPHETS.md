# Пророки: как создать свою модель на Kenga

Пророк — модель, у которой тело лежит в файле, которая учится на проверенном
опыте и которую второй запуск поднимает тем же телом. Это не «чат-LLM» и не
обёртка над чужим API. Пророк создаётся на самом языке: `Memory`, тензоры,
tape и `save_mind`/`load_mind` — обычные функции Kenga, новых ключевых слов
для этого нет (см. `docs/SPEC.md` §12).

Слово «Пророк» (Prophet) — потому что основная операция — `foresee`:
предсказать наблюдение до того, как оно пришло, и удивиться (`surprise`),
если не совпало. Учится модель только на удивлении.

## 1. Семейства Пророков, которые уже есть

| Семейство | Что это | Где | Статус |
|---|---|---|---|
| Prophet Memory | world-model (residual MLP) + эпизодическая память + сон (`consolidate`) | `examples/prophet.kenga`, `examples/ml/living_prophet.kenga`, `examples/telegram_bot.kenga` | ✅ lite, more-VM, bc→C — без Rust |
| Prophet-LM | модели, которые пишут Kenga-код: Pico M0/M1/M2, `kenga_lm`, `kenga_charlm`, birth → `kenga_born.kenga` | `examples/ml/`, `docs/PICO_PROPHET.md`, `docs/KENGA_LM.md` | ✅ pass-rate 9/9 (M0), 81 % token-acc (M2) |
| Prophet-Embed / Z-Embed | 42M byte-энкодеры для поиска и STS, instruct-протокол как у Giga | HF `GermannM/kenga-embed-prophet-instruct`, `GermannM/kenga-embed-z` | ✅ веса на HF; тренер пока PyTorch (лаба, не в репо) |
| Prophet 400M | спека промышленного Пророка: Z-ёмкость + смесь проверенных программ | `examples/ml/prophet_400m.kenga` | ◐ спека есть, тренер в лабе |

## 2. Контракт Пророка

Что бы ни лежало внутри (16-d память бота или 42M энкодер), Пророком в Kenga
называется то, что выполняет пять условий:

1. **Тело — файл.** `.km` (`save_mind` / `load_mind`, формат `KENGA_MIND 1`),
   `.kt` (`save_tensor` / `load_tensor`), для HF-выкладки — `pytorch_model.bin`.
   Веса в git не коммитятся (`docs/REPO.md`).
2. **Паспорт.** `mem_stats(mind)` → `[episodic, core, locked, …]`; у энкодеров
   на HF — `config.json` с размерами и метриками. Пророк без паспорта — не Пророк.
3. **Учитель — проверенный опыт, не мнение.** Для мира — `surprise(pred, obs)`;
   для кода — оракул `kenga-lite` (программа скомпилировалась и дала нужное
   число); для бота — сам чат (`minds/tg_pairs.txt`). Loss = pass-rate, не perplexity.
4. **Второй запуск продолжает то же тело.** `file_exists(path)` → `load_mind`,
   иначе `memory_config`. Никаких «обучи заново с нуля».
5. **Вызывается из Kenga как функция.** `foresee` / `recall` / `predict`
   возвращают `list`; программа сама решает, что с этим делать.

## 3. Минимальный Пророк (проверено на `kenga-lite`)

```kenga
// Тело в файле, второй запуск продолжает то же тело.
fn main() -> i64 {
    let path = "minds/my_prophet.km";
    let mind: Memory = memory_config(10, 64, 32); // thr=0.10, episodic 64, core 32
    if file_exists(path) {
        mind = load_mind(path);
        println("continue");
    } else {
        println("born");
    }
    let obs = [2, 2, 9];
    let pred = foresee(mind, obs);      // предсказание до наблюдения
    let s = surprise(pred, obs);        // RMSE → f64
    if s > 0.2 {
        remember(mind, obs, s);         // в эпизодический буфер
    }
    consolidate(mind);                  // сон: эпизоды → core + EWC-lite
    println(mem_stats(mind));
    save_mind(mind, path);
    return 0;
}
```

Первый запуск печатает `born` и `[0, 1, 10]`, второй — `continue` и
`[0, 2, 10]`: тело выросло и осталось тем же файлом.

Полная версия с обучаемой world-model (`remember_next`, `learn`, `predict`,
паспорт на диске, переобучение на оракуле): `examples/ml/living_prophet.kenga`.
API на lite целиком: `docs/PROPHET_LITE.md`.

## 4. Пророк, который пишет Kenga

Второе семейство — генераторы кода. Рецепт один и тот же на всех размерах:

1. Корпус — `kenga/` + `examples/` + `examples/selfhost/`. Свой, без лицензий.
2. Оракул — `bootstrap/bin/kenga-lite.exe`: сгенерированная программа обязана
   скомпилироваться, содержать `fn main` и напечатать ожидаемое значение.
3. Метрика — pass-rate на оракуле (`scripts/pico-birth.sh`), не perplexity.
4. Модель — что угодно: суффиксный матчер без параметров (Pico M0),
   линейный классификатор (M2, `tools/train_m2.py`), decoder на tape
   (`examples/ml/kenga_lm.kenga`, D=32 L=2, native C за 0.3 с).

Лестница размеров и что даёт каждая ступень — `docs/KENGA_LM.md`;
почему маленькая модель на своём корпусе воспринимается как 27B —
`docs/NEUROMODEL_27B.md`.

## 5. Энкодеры Z-Embed и Prophet-Embed (42M)

Первые Пророки, которые можно скачать и проверить снаружи. Оба — 42M
двунаправленных энкодера над сырыми UTF-8 байтами (V=256, без токенизатора,
D=768, L=8, rank 192/512, mean-pool). Prophet-instruct принимает те же
instruct-префиксы, что Giga-Embeddings; Z — сырой текст.

Что измерено (03.09.2026):

| Модель | Параметры | Holdout 24 тройки (12 EN + 12 RU) | RuSTS test, Spearman |
|---|---:|---:|---:|
| Giga-Embeddings-instruct | 480M | 21/24 (10 EN, 11 RU) | 0.803 |
| Z-Embed (`kenga-embed-z`) | 42M | 23/24 (12 EN, 11 RU) | 0.372 |
| Prophet-Embed-instruct | 42M | 22/24 (10 EN, 12 RU) | 0.442 |

Читать честно: holdout — 24 ручные тройки с ловушками (Эльсинор без «Гамлета»,
H2O против H2O2, Лев Толстой против Лео Штрауса), не ruMTEB. На нём 42M
обходит 480M. На RuSTS 42M пока вдвое хуже Giga. Полный `MTEB(rus, v1)` не
гонялся — нет `pytrec_eval` в тренировочном окружении; официальный прогон
за командой Giga.

Что здесь от языка: byte-level (нет словаря — нет проблемы кириллицы),
контракт §2 (тело + паспорт `config.json`), holdout-оракул вместо мнения.
Сам тренер (`exp_EMBED.py`) — PyTorch в лабе `z-system`; в репо не входит.

## 6. Дорога к «нормальному» Пророку и «нормальному» Z

Сегодня скачать и поговорить можно только с `GermannM/Kenga` и
`GermannM/Kenga-Trained`. Энкодеры не отвечают текстом. «Нормальный» Пророк —
тот, с которым говорят, как с этими двумя, но который лёгкий и живёт
по контракту §2.

| Ступень | Что | Условие готовности |
|---|---|---|
| П0 ✅ | Энкодеры 42M на HF, instruct-протокол Giga | holdout 24 троек ≥ Giga — есть |
| П1 ◐ | Prophet-LM decoder 42M–400M на корпусе Kenga + русские пары бота; спека `examples/ml/prophet_400m.kenga` | pass-rate на оракуле ≥ Pico M0 на held-out программах; на 1660 6 GB — один прогон, без AdamW на 480M |
| П2 ⬜ | Instruct: SFT на парах `minds/tg_pairs.txt` + диалоги с `Kenga-Trained`; загрузчик `modeling_*.py` с `chat()` на HF | второй запуск продолжает тот же `.pt`; ответы проходят тот же holdout |
| П3 ⬜ | Сравнение с 9B / 27B | на нашей задаче: pass-rate программ Kenga и retrieval-holdout. Не общий chit-chat: 42M не будет знать столько фактов, сколько 27B, и мы этого не обещаем |

Язык при этом не меняется: ни П1, ни П2 не требуют новых ключевых слов.
Отдельная конструкция `prophet Name { … }` — кандидат, а не план: по §1 SPEC
цена считается в токенах и появится только после того, как рецепт из §3
пройдёт всю лестницу self-host (`docs/SELFHOST.md`).

## 7. Что не коммитим

Веса (`.km` живых умов, `.kt`, `.pt`), корпуса, holdout-тройки и тренеры лабы.
Публичное — язык, рецепт, оракул и паспорт. Веса — на Hugging Face.