# Kenga Language Changelog

Новые конструкции, вошедшие в мерж `b6fe466` (ветка desktop/match/enum поверх main).
Источники: `git log --oneline -30`, diff `src/parser.rs`, `src/ast.rs`, `tests/lang.rs`.

Легенда статусов: **parser** — принимает ли Rust-парсер; **VM** — lowering в байткод
(`src/compiler.rs`, исполнение через `kenga::vm`); **emit-c** — C99-бэкенд
(`src/codegen.rs`); **kenga-lite** — самохостинг (`examples/selfhost/kenga_lite.kenga`
→ `bootstrap/kenga_lite.c`).

## Сводная таблица

| Конструкция | Пример | Commit | parser | VM | emit-c | kenga-lite |
|---|---|---|---|---|---|---|
| match-оператор (arms + bindings) | см. ниже | `07c20b8`, `4d806ae` | OK | ✗ (`compiler.rs`: «match bytecode lowering is not implemented») | OK (`d17ba9f`) | ✗ |
| Disambiguation scrutinee `{` | `match x { ... }` | `d2e8eab` | OK | — | — | — |
| return без значения в arm | `=> { return; }` | `a603207` | OK | ✗ | OK | ✗ |
| enum с payload | см. ниже | `8d70fbe`, `4e2217c` | OK | ✗ (декларация игнорируется) | OK (`390b170`) | ✗ |
| Вариантные литералы Some/None | `Some { value: n }`, `None` | `6e9ba7e` | OK | ✗ | OK | ✗ |
| union-типы | `union U { x: i64 }` | `fb50182` | OK (как struct) | как struct | как struct | ✗ |
| Bounded arrays | `let xs: array<i64, 8>` | `fb50182` | OK (→ `Type::List`) | как list | как list | ✗ |
| Generic fn pointers | `ptr<fn(i64) -> i64>` | `dbc6ec4` | OK (→ `Type::Named`) | как named | вызов не эмитится | ✗ |
| Generic Option | `Option<i64>` | `dbc6ec4` | OK (→ `Type::Named`) | как named | tagged payload | ✗ |
| Квалифицированные типы | `Renderer::Framebuffer` / `Renderer.Framebuffer` | `a78f239` | OK | как named | как named | ✗ |
| Ссылки `&` / `&mut self` | `fn m(&mut self)` | `a78f239` | OK (ABI-transparent) | receiver = i64 | handle | ✗ |
| Stepped ranges | `for i in 0..10 step 2` | `aec6185` | OK | обычный range | OK | ✗ |
| Compound assignments | `x += 1; y *= 2;` | `1ed6871` | OK (+,-,*,/) | OK | OK | частично |
| Desktop polling (`while let`) | `while let Some(e) = poll()` | `1ed6871` | OK (sugar) | условие = `poll()` | то же | ✗ |
| Import aliases | `import "gfx.kenga" as Gfx;` | `02a6920` | OK (валидация) | import плоский | — | ✗ |
| Скалярные касты | `value as float` | `a78f239` | OK (аннотация съедается) | coercion в VM | coercion | ✗ |
| const-элементы | `const MAX = 10;` | `9f25d36` | OK | ✗ | ✗ | ✗ |
| impl-блоки + методы | `impl R { fn draw(&mut self) }` | `9f25d36` | OK | методы как fn | как fn | ✗ |
| Вызовы методов через точку | `obj.draw(x);` | `9f25d36` | OK → `Call` | как вызов fn | как вызов fn | ✗ |
| Static calls через `::` | `Type::make(3);` | `9f25d36` | OK → `Call` | как вызов fn | как вызов fn | ✗ |
| Опциональные `;` | `let x = 1` (без `;`) | `9f25d36` | OK (**laxity!**) | — | — | источник бага juxtaposition |
| Port I/O intrinsics | `asm_inb(port)`, `asm_outw(p, v)` | `0e258c2` | OK (builtin) | ✗ | inline asm x86 | ✗ |
| Hex/bin литералы | `0x20`, `0b1010` | `0e258c2` | OK | OK | OK | ✗ |

## Примеры синтаксиса

### match с arms/bindings (`07c20b8`, `4d806ae`; codegen `d17ba9f`, `390b170`)

```kenga
enum Option {
    Some { value: i64 },
    None,
}

fn classify(o: Option) -> i64 {
    match o {
        Option::Some { v } => {
            return v * 2;
        }
        None => {
            return 0;
        }
    }
}
```

Паттерны: `_` (wildcard), `Path::Variant { b1, b2 }` (bindings). Разделитель arm —
необязательная `,`. Scrutinee может быть только идентификатором или выражением;
`{` после него трактуется однозначно как начало arms (`d2e8eab`). `return;` без
значения внутри arm разрешён (`a603207`). В байткод **не** понижается:
`Stmt::Match` даёт ошибку «match bytecode lowering is not implemented».

### enum с payload (`8d70fbe`, `4e2217c`)

```kenga
enum Shape {
    Point,
    Circle { r: f64 },
    Rect { w: i64, h: i64 },
}
```

Вариант без payload и с payload `{ field: Type }`. emit-c хранит payload в
слотах `KVal.u.payload[0..3]` (`kval_payload_i64`), тег — по имени варианта.

### Вариантные литералы (`6e9ba7e`)

```kenga
let s = Option::Some { value: 5 };   // квалифицированная форма
let n = None;                        // голый None
let t = Some { value: 7 };           // неквалифицированная форма
```

### union / bounded arrays / generic указатели (`fb50182`, `dbc6ec4`)

```kenga
union Pixel { r: i64, g: i64 }          // парсится как struct (portable ABI)

fn sum(xs: array<i64, 8>) -> i64 { ... }   // → list фиксированной ёмкости
fn apply(f: ptr<fn(i64) -> i64>, x: i64) -> i64 { ... }
fn maybe(v: Option<i64>) -> Option<i64> { ... }
```

### Квалифицированные типы, ссылки, касты (`a78f239`)

```kenga
fn load(fb: Renderer::Framebuffer) -> i64 { ... }   // тоже Renderer.Framebuffer
fn push(&mut self, v: i64) -> i64 { ... }           // method receiver
let y = x as float;                                  // скалярный каст (аннотация)
```

### Stepped ranges (`aec6185`)

```kenga
for i in 0..20 step 3 {
    total = total + i;
}
```

### Compound assignments (`1ed6871`)

```kenga
acc += delta;
acc -= 1;
acc *= k;
acc /= 2;
```

Работают для целей `Name`, `Index[..]`, `Field..`.

### Desktop polling (`1ed6871`)

```kenga
while let Some(event) = poll() {
    handle(event);
}
```

Sugar: паттерн `Some(binding)` съедается, условием становится `poll()` —
option-подобный poll в текущем ABI — скалярное условие.

### Import aliases (`02a6920`)

```kenga
import "renderer.kenga" as Renderer;
```

Импорты сейчас флэттятся драйвером; алиас валидируется и сохраняется в AST.

### const / impl / методы (`9f25d36`)

```kenga
const MAX_LEVEL = 9;

impl Counter {
    fn bump(&mut self) -> i64 {
        self.n += 1;
        return self.n;
    }
}
// вызов: c.bump();  или static: Vec::make(3);
```

### Port I/O intrinsics (`0e258c2`)

```kenga
@intrinsic fn kf_get_boot_info() -> i64;

let status: i64 = asm_inb(0x60);      // inb/inw/inl — 1 аргумент
asm_outb(0x20, 0x11);                 // outb/outw/outl — порт, значение
```

emit-c генерирует inline asm с constraints `'a'/'Nd'`; `@intrinsic`-функции
эмитятся как extern-прототипы с `k_`-mangling.

## Регрессия: ослабление парсера (важно для Factory)

`optional_semicolon` (`9f25d36`) сделало `;` необязательным, из-за чего блок
парсится «по одному выражению»: `return 5 5;` принимается как два стейтмента
(`return 5;` затем голое `5`), а `return a b + 9;` молча компилируется
самохостным **kenga-lite** (`bootstrap/kenga_lite.c`). Регрессионные тесты:
`tests/parser_laxity.rs` (помечены `#[ignore]` до починки).
