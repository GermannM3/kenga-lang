# Kenga Resource Spec — пулы, размещение, бюджеты

Статус: **DRAFT** (дизайн-док, реализация не начата).
Связанное: `docs/LANGUAGE.md`, `docs/ROADMAP.md`, `docs/FACTORY_V3_NOTES.md`, `docs/GENESIS_V0.md`.
Parse-only скелеты тестов уже лежат в `tests/lang.rs` (помечены `#[ignore = "TODO(parser)"]`).

## 1. Мотивация

Источник идей — проект **FreeToken** (edge-native MoE serving): единый пул
CPU+GPU+RAM, динамическое размещение и кэширование экспертов,
перераспределение памяти между KV-cache и экспертами без reload,
bandwidth-adaptive execution, semantic checkpoints.

Отличие наше от их подхода принципиальное:

* **FreeToken**: runtime-слой *поверх чужих моделей* — оркестратор живёт вне
  модели, модель ресурсами не управляет.
* **Kenga**: это **конструкции самого языка**, а рантайм написан на самом
  Kenga (правило KengaOS: C — только `bootstrap/hardware`; вся новая
  функциональность пишется на Kenga). Нейромодель получает *языковой*
  контроль над ресурсами: factory сможет генерировать `pool`/`place`/`budget`
  так же естественно, как сейчас генерирует `fn`/`for`.

Два независимых датчика давления:

| Датчик | Что меряет | Кормит конструкции |
|---|---|---|
| **hardware-state** | free RAM, CPU load, link bandwidth, GPU | `pool`, `place`, `budget`, `adapt`, `checkpoint` |
| **spectral/model-state** | surprise / энтропия / плотность спектра активаций Prophet | зарезервированный `grow <zstate> by <k>` — задел под будущий **Z-Capacity** (`grow zspace by k`) |

Язык — точка встречи этих двух давлений: модель *просит* (`grow`),
железо *отвечает* (датчики), менеджер на Kenga *решает*.

## 2. Синтаксис-эскиз

Новые жёсткие ключевые слова — минимум: `pool`, `place`, `budget`, `adapt`,
`checkpoint`, `grow`, `pin`, `evict`, `rest`, `every`, `by`; слово `on`
переиспользуется из events. Метки полей (`cpu`, `ram`, `gpu`, `auto`,
`none`, `local`, `remote`, `min_bw`, `prefer`, `semantic`) — **контекстные
идентификаторы**, не ключевые слова: меньше терминалов — меньше раздувание
кодека (см. `docs/FACTORY_V3_NOTES.md` про vocab).

Размеры пишутся литералами `16GiB` / `512MiB` / `64KiB` (лексер выдаёт
один токен размера; см. открытые вопросы §5.1).

### 2.1 `pool` — ресурсный домен

```kenga
pool <name> { cpu: auto|<n>, ram: <size>, gpu: none|auto|<n> }
```

Пул — именованный домен ресурсов с декларативными лимитами: `auto` =
рантайм сам спрашивает железо через bootstrap-датчики, `none` = ресурс
недоступен, число = жёсткий предел. Пул ничего не выделяет при объявлении —
это единица учёта для `place`/`budget`. Размещение в чужом пуле (другая
машина) — см. §5.3.

```kenga
// Инференс MoE-модели на десктопе без GPU
pool desk {
    cpu: auto,
    ram: 24GiB,
    gpu: none           // считаем только на CPU
}

pool peer {             // вторая машина в локалке
    cpu: 8,
    ram: 16GiB,
    gpu: auto
}

fn main() -> i64 {
    println("pools declared");
    return 0;
}
```

### 2.2 `place` — декларативное размещение состояния

```kenga
place <name> on <pool>(<budget>) pin|evict
```

Декларативно связывает именованное состояние программы (tensor, Memory,
список) с пулом и бюджетом внутри него. `pin` — состояние нельзя вытеснять
(горячий KV-cache активной сессии); `evict` — можно выгружать/подгружать по
давлению (холодные эксперты MoE). Аргумент в скобках — либо литерал размера,
либо ссылка на слот бюджета (`бюджет.слот`, §2.3). Повторный `place` того же
имени на другой пул — **миграция**: рантайм переносит содержимое, а не
делает reload с нуля.

```kenga
pool home { cpu: auto, ram: 24GiB, gpu: none }
pool lab  { cpu: 16,   ram: 64GiB, gpu: auto }

let w = t_from([8, 4096, 4096], weights);  // банк экспертов слоя

place w on home(4GiB) evict;               // холодные эксперты — eviction

fn migrate_to_lab() -> i64 {
    // перенос состояния между машинами: тот же place, другой пул —
    // runtime делает move, а не пересчёт с нуля
    place w on lab(12GiB) pin;
    return 0;
}

fn main() -> i64 {
    return migrate_to_lab();
}
```

### 2.3 `budget` — слоты с перераспределением

```kenga
budget <name> { <slot>: <size>, ..., other: rest }
```

Именованная раскладка памяти внутри одного пула. `rest` — весь остаток пула,
отдаётся под всё прочее (scratch, новые размещения). Главная идея,
перенесённая из FreeToken: **перераспределение между KV-cache и экспертами
без reload**. Слот — это обычное присваиваемое имя: `moe.kv = 7GiB` меняет
границу слотов, рантайм сдвигает память, содержимое остальных слотов не
инвалидируется. Слот привязывается к состоянию через `place` со ссылкой
`бюджет.слот`.

```kenga
budget moe {
    hot_experts: 10GiB,
    kv:          6GiB,
    other:       rest         // остаток пула — под что угодно
}

pool desk { cpu: auto, ram: 24GiB, gpu: none }

let experts = load_tensor("experts.kt");
let kv      = memory_config(64, 2048, 16);

place experts on desk(moe.hot_experts) evict;
place kv      on desk(moe.kv) pin;

fn main() -> i64 {
    // контекст вырос: отдаём KV ещё гигабайт за счёт cold-экспертов.
    // Без reload: runtime двигает границу слотов, pin-содержимое живо.
    moe.kv = 7GiB;
    return 0;
}
```

### 2.4 `adapt` — bandwidth-adaptive исполнение

```kenga
adapt <link> { min_bw: <size>, prefer: local|remote }
```

Политика для именованной связи между пулами/узлами (связи обнаруживает
рантайм через bootstrap-датчик link-bw). `min_bw` — минимальная пропускная
способность, при которой удалённое исполнение допустимо; `prefer` — куда
стараться ставить вычисления (к данным или данные к коду). Если фактический
bw проседает ниже `min_bw`, рантайм сам переключает план исполнения
(например, тянет эксперта локально), а текст программы не меняется —
вызовы остаются теми же.

```kenga
pool laptop { cpu: 8,  ram: 16GiB,  gpu: none }
pool rack   { cpu: 64, ram: 256GiB, gpu: auto }

// связь laptop->rack найдена рантаймом; веса огромные —
// считаем там, где они лежат
adapt laptop_to_rack {
    min_bw: 80MiBps,
    prefer: remote
}

let bank = load_mind("minds/experts.km");
place bank on rack(48GiB) pin;

fn main() -> i64 {
    // ночной прогон: если канал просядет ниже min_bw,
    // runtime начнёт подтягивать работу на laptop сам
    let traj = unroll(bank, [1, 0], 512);
    return len(traj);
}
```

### 2.5 `checkpoint semantic` — семантические контрольные точки

```kenga
checkpoint semantic "<path>" every <n>
load checkpoint "<path>"
```

Декларация периодического сохранения **семантического** состояния — не
побайтового дампа: значения и структура (tensor → как `save_tensor`,
Prophet mind → формат `.km`, сводка KV-контекста), без адресов. Благодаря
этому восстановление работает на другой машине и после eviction-reload.
`<n>` — период в шагах основного цикла (единицы — см. §5.4). Чтение —
выражение `load checkpoint "<path>"`: подтягивает последний снапшот, после
чего программа продолжает с того же логического места.

```kenga
pool desk { cpu: auto, ram: 24GiB, gpu: none }

let ctx = memory_config(64, 1024, 16);
checkpoint semantic "sessions/chat_a" every 200;

fn main() -> i64 {
    for step in 0..10000 {
        learn(ctx, [step, 1], [step + 1, 1]);
        if step == 500 {
            break;              // машина уходит в сон
        }
    }
    save_mind(ctx, "minds/chat_a.km");
    return 0;
}

// ---- продолжение на второй машине ----
// load checkpoint "sessions/chat_a";
// for step in 500..10000 { learn(ctx, ...) }  // то же логическое место
```

### 2.6 `grow` — зарезервировано (spectral pressure, Z-Capacity)

```kenga
grow <zstate> by <k>
```

**Зарезервированная конструкция**: фаза A узнает токены, семантика
приходит в фазе C. Идея: spectral/model-state датчик — рост `surprise`,
энтропии предсказаний, плотности спектра активаций Prophet — сигнал, что
ёмкости пространства состояний Z не хватает. Тогда модель сама *просит*
языком расширить именованное спектральное состояние в `k` раз:
`grow zspace by 2`. Решение остаётся за рантаймом: давление hardware-state
(нет RAM) даёт отказ, который модель читает как событие. Это замкнутый
контур Z-Capacity: spectral pressure ↑ → `grow` → capacity ↑ → surprise ↓,
а при упоре в железо — eviction/миграция.

```kenga
// ЗАРЕЗЕРВИРОВАНО: парсер фазы A примет, семантика — фаза C
fn main() -> i64 {
    let m = memory();
    learn(m, [1, 0], [0, 1]);
    let s = surprise(m, [9, 9]);    // спектральный датчик давления
    if s > 3.0 {
        grow zspace by 2;           // просим удвоить ёмкость Z
    }
    return 0;
}
```

## 3. Соответствие FreeToken → Kenga

| Механизм FreeToken | Конструкция Kenga | Чем отличается |
|---|---|---|
| Единый пул CPU+GPU+RAM | `pool <name> { … }` | пул — синтаксическая конструкция с декларативными лимитами, а не конфиг оркестратора; учёт ведёт рантайм, написанный на самом Kenga |
| Динамическое размещение/кэширование экспертов | `place <x> on <pool>(<b>) pin\|evict` | место и режим eviction заданы в исходнике; генерирующая модель пишет `place` так же, как `fn` |
| Перераспределение памяти KV-cache ↔ эксперты без reload | `budget { …, other: rest }` | перебалансировка — присваивание слоту нового размера (`moe.kv = 7GiB`); без рестарта, без инвалидации содержимого |
| Bandwidth-adaptive execution | `adapt <link> { min_bw, prefer }` | политика видна компилятору и модели в тексте программы; переключение плана исполнения — дело рантайма, код не меняется |
| Semantic checkpoints | `checkpoint semantic … every n` + `load checkpoint` | checkpoints первого класса: период — декларация, восстановление — выражение; формат семантический (значения, не адреса) |
| (нет прямого аналога) | `grow <zstate> by <k>` | наш собственный задел: spectral pressure → языковой запрос ёмкости под Z-Capacity; у FreeToken модель ресурсов не запрашивает |

## 4. План реализации по фазам

### Фаза A — спека + парсер

**Lexer** — новые токены (`src/token.rs`):

```rust
// keywords
Pool, Place, Budget, Adapt, Checkpoint, Grow,
Pin, Evict, Rest, Every, By,
// размеры: 16GiB / 512MiB / 64KiB -> один токен, байты
Size(u64),
// `On` уже есть (events) — переиспользуем в `place x on pool(...)`;
// auto/none/local/remote/semantic/min_bw/prefer — контекстные Ident
```

Дизамбигуация: `on` начинает event-handler только в начале statement;
внутри `place` он всегда следует за идентификатором — конфликтов нет.

**AST** (`src/ast.rs`) — новые узлы:

```rust
Item::Pool(PoolDecl { name: String, cpu: Cap, ram: u64, gpu: Cap }), // Cap::Auto|Count(n)|None
Item::Budget(BudgetDecl { name: String, slots: Vec<(String, SlotSize)> }), // SlotSize::Bytes(u64)|Rest
Stmt::Place { target: String, pool: String, budget: BudgetArg, mode: PlacementMode }, // Pin|Evict
Stmt::Adapt { link: String, min_bw: u64, prefer: Pref },             // Pref::Local|Remote
Stmt::Checkpoint { path: String, every: u64 },
Expr::LoadCheckpoint(String),
Stmt::Grow { state: String, amount: Box<Expr> },                     // reserved
```

**Тесты** — в `tests/lang.rs` добавлены 7 parse-only скелетов
(`resource_*`, каждый помечен `#[ignore = "TODO(parser): …"]`, общий хелпер
`parses`):

| Тест | Покрывает |
|---|---|
| `resource_pool_decl` | `pool` с auto/none/auto-числом, gpu |
| `resource_place_pin_evict` | `place … on …(<size>) pin` / `evict`, повторный place = миграция |
| `resource_budget_slots` | `budget` со слотами + `other: rest` |
| `resource_budget_rebalance` | присваивание слоту `moe.kv = 7GiB` |
| `resource_adapt_link_policy` | `adapt` с `min_bw`/`prefer` |
| `resource_checkpoint_roundtrip` | `checkpoint semantic … every` + `load checkpoint` |
| `resource_grow_zspace_reserved` | `grow zspace by 2` |

Фрагмент (полные версии — `tests/lang.rs`, конец файла):

```rust
#[test]
#[ignore = "TODO(parser): pool/budget/place"] // docs/KENGA_RESOURCE_SPEC.md, фаза A
fn resource_budget_rebalance() {
    parses(
        r#"
        budget moe {
            hot_experts: 10GiB,
            kv: 6GiB,
            other: rest
        }
        pool desk { cpu: auto, ram: 24GiB, gpu: none }
        place experts on desk(moe.hot_experts) evict;
        place kv on desk(moe.kv) pin;
        fn main() -> i64 {
            moe.kv = 7GiB;
            return 0;
        }
        "#,
    );
}
```

Критерий выхода фазы A: `cargo test --test lang` зелёный со снятыми
`#[ignore]` (все 7 программ парсятся), codegen пока может отвергать эти
узлы явной ошибкой «not yet supported».

### Фаза B — семантика на Kenga

Никакой логики в C (правило KengaOS). В `bootstrap/hardware/` — только
**read-only датчики**; менеджер пулов — Kenga-программа
(`kenga/resource/poolman.kenga`) поверх существующих интринсиков и events
(`on "…"` / `emit` / `pump` уже есть, `@intrinsic`-FFI уже есть — см.
тест `intrinsic_ffi_emit_c`).

Минимальные syscall-хуки bootstrap (только query, без мутаций):

```c
/* bootstrap/hardware/sensors.c — только чтение, без политики */
int64_t k_sys_free_ram (int32_t node);           /* байт, -1 = неизвестно */
int64_t k_sys_total_ram(int32_t node);           /* байт */
double  k_sys_cpu_load (int32_t node);           /* 0.0..1.0, NaN = неизвестно */
double  k_sys_link_bw  (int32_t a, int32_t b);   /* байт/с, NaN = связи нет */
int64_t k_sys_gpu_free (int32_t node);           /* байт, -1 = нет GPU */
```

Обёртки на стороне Kenga:

```kenga
@intrinsic fn kf_free_ram(node: i64) -> i64;
@intrinsic fn kf_link_bw(a: i64, b: i64) -> f64;
@intrinsic fn kf_cpu_load(node: i64) -> f64;
```

Менеджер пулов на Kenga: таблица пулов/бюджетов/размещений, цикл давления
через события — `on "hw_pressure"(stats)` читает датчики, решает кого
evict (по меткам `pin`/`evict`), исполняет ребаланс слотов и миграции;
`emit("grow_denied", reason)` при отказе в `grow`. Всё это — обычный
Kenga-код, тестируемый на lite/full VM с **fake-датчиками**
(детерминированные значения вместо `kf_*`).

### Фаза C — связка с Prophet/Z

* **Factory** (`docs/GENESIS_V0.md`, `tools/corpus_factory.py`): новая
  категория программ `resource` рядом с `arith/bind/chain/loop/rec` —
  короткие verified-программы с `pool`+`place`+`budget`+`checkpoint`.
  Oracle прежний: компиляция + запуск, но датчики подменяются
  детерминированным fake-hw (pass-rate воспроизводим). Слова
  `pool/place/budget/adapt/checkpoint/grow/rest/every` попадают в корпус —
  по `docs/FACTORY_V3_NOTES.md` они уже представимы буквенными
  fallback-BPE, терминалы — вопрос компактности, не корректности.
* **Нейромодель** начинает писать программы-распорядители: «разместил →
  проверил давление → перебалансировал», причём это те же `fn`+events,
  которые она уже умеет.
* **`grow zspace` ↔ Z-Capacity**: spectral-state публикует давление
  (`surprise` уже есть в Prophet API); `grow <zstate> by <k>` — языковой
  запрос ёмкости. Z-Capacity-менеджер (следующий этап) сопоставляет
  spectral pressure с hardware pressure: есть ресурсы — растим, нет —
  эвикшн/миграция/отказ-событие. Контур: рост surprise → grow → capacity ↑
  → surprise ↓; упор в железо виден модели как события, а не как тишина.

## 5. Открытые вопросы

1. **Типы размеров.** Литералы `16GiB` десугарятся в `i64`-байты (один
   токен `Size(u64)`), отдельного типа `bytes` нет — минимальное вторжение
   в систему типов. Риск: в полях пула тогда синтаксически валидно и
   голое `24000000000`. Запрещать ли голые числа в полях `pool`/`min_bw`
   на уровне парсера?
2. **Исчерпание пула.** Ошибка времени выполнения или eviction-коллбек?
   Рабочий гибрид: `evict`-состояния вытесняются молча; конфликт на
   `pin` → событие `"pool_exhausted"`, и лишь если его никто не обработал
   — runtime error. Не решено: кто выбирает жертву при нескольких
   evict-кандидатах (LRU? приоритет в place?) и кто платит за неудачную
   миграцию (откат или partial state).
3. **Права/безопасность.** Какая модель может делать `place` на чужой пул?
   Дефолт-предложение: пулы принадлежат программе-владельцу; кросс-пульный
   (тем более кросс-машинный) place требует handshake — владелец должен
   подписаться `on "place_request"(req)` и ответить, иначе deny-by-default.
   Отдельный вопрос: имеет ли право generated-код (factory output) делать
   `place`/`grow` без участия человека — вероятно, capability передаётся
   явно при запуске программы.
4. **Единицы `every <n>`.** Шаги основного цикла VM, миллисекунды или
   итерации внешнего цикла программы? На native-C пути (emit-c) понятие
   «шаг VM» исчезает — возможно, нужен явный `checkpoint_now()` вызов.
5. **`prefer: remote` при провале bw.** Кого двигать: вычисление к данным
   или данные к вычислению — и должно ли это быть видно языку (событие
   `"plan_switched"`), чтобы модель могла реагировать?
