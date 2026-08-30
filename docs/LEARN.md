# Учим Kenga за один вечер

Не по наитию — по шагам. Друзьям проще начать **без Rust**: бинарник с Releases или `bootstrap/`. Полный `cargo install` — если хочешь GPU/legacy path.

```bat
bootstrap\build.cmd
bootstrap\bin\kenga-lite.exe version
```

или с Releases: `kenga version` (3.13+).

## 0. Demo

```bash
kenga demo
kenga about
```

## 1. Hello

`examples/hello.kenga`:

```kenga
fn main() -> i64 {
    println("hello from kenga");
    return 0;
}
```

```bash
kenga run --lite examples/hello.kenga
```

## 2. Условия и циклы

Смотри `examples/showcase.kenga` и `examples/selfhost/for_lite.kenga`:

- `if cond { … } else if … else { … }`
- `while cond { … }`
- `for i in 0..n { … }` / `for x in xs { … }`
- `break` / `continue`
- `match x { 0 => { … } _ => { … } }` — i64 везде; строки — на more (`examples/selfhost/match_str.kenga`)
- `slice` / `index_of` / `starts_with` / `split` — `examples/selfhost/str_lab.kenga`
- `map_new` / `map_get` / `map_set` / `map_has` / `json_set` — more; лаба `map_lab.kenga`
- `&&` `||` `!` `%` `typeof` `to_str` (lite, `more`, lowerers, `bc_src_c`)
- `print` без `\n`, `sleep_ms` (lite, `more`; `lower_c` / `lower_kv` / `bc_src_c`)
- `learn` / `predict` / `unroll` / `remember_next` на `more` VM (хост — lite builtins)
- `foresee_n` на lite и `more` (`examples/unroll.kenga` без Rust)
- `now_ms` — часы, не заглушка (lite, `more`, `lower_c` / `lower_kv` / `bc_src_c`); комментарии `//` и `/* … */`

## 3. Списки и struct

`examples/selfhost/lists_lite.kenga`, `examples/selfhost/struct_lite.kenga`:

```kenga
let xs = [1, 2, 3];
xs = push(xs, 4);
println(len(xs));

struct Point { x, y }
let p = Point { x: 3, y: 4 };
println(p.x);
```

## 4. События агента

`examples/agent.kenga` — `on` / `emit` / `pump` / `pending`.

```bat
kenga run --lite examples\agent.kenga
```

## 5. Память и world-model

```bash
kenga run --lite examples/ml/world_model.kenga
kenga run --lite examples/ml/surprise_gate.kenga
kenga chat --lite minds/agent.km
```

API: `memory`, `learn`, `predict`, `unroll`, `surprise`, `consolidate` — см. `docs/LANGUAGE.md`.

## 6. Тензоры и multimodal

```bash
kenga run --lite examples/ml/tensor_core.kenga
kenga run --lite examples/ml/train_sgd.kenga
kenga run --lite examples/ml/vision_ppm.kenga
kenga run --lite examples/ml/fusion.kenga
kenga run --lite examples/ml/living_multimodal.kenga
kenga run --lite examples/ml/kenga_mm_lm.kenga
bootstrap\bin\kenga-lite.exe run kenga\compiler\more.kenga
```

`more.kenga` на своей VM: `examples/selfhost/tensor_more.kenga`, `tape_more.kenga`, `vision_more.kenga`, `print_more.kenga`, `learn_more.kenga`, `unroll_more.kenga`, `foresee_n_more.kenga`, `examples/prophet.kenga`, `examples/ml/living_multimodal.kenga`, `examples/ml/kenga_net.kenga`, `examples/ml/word_lm.kenga`.

`kenga_mm_lm` — картинка + звук → подпись (`kenga vidit … i slyshit ton`). Это сид большой модели, не CLIP.

## 7. Без Rust (lite + свой codegen)

```bat
bootstrap\build.cmd
kenga run --lite examples\selfhost\hello_lite.kenga
kenga run --lite examples\selfhost\lists_lite.kenga
kenga run --lite examples\selfhost\struct_lite.kenga
kenga run --lite examples\selfhost\float_lite.kenga
kenga run --lite examples\selfhost\elif_lite.kenga
kenga run --lite examples\selfhost\for_lite.kenga
kenga run --lite examples\selfhost\str_lab.kenga
kenga run --lite examples\selfhost\map_lab.kenga
kenga run --lite examples\selfhost\match_str.kenga
kenga run --lite kenga\compiler\lite.kenga
kenga run --lite examples\agent.kenga
```

Компилятор и VM на Kenga: `kenga/compiler/more.kenga` — birth→24, XOR, `c_seed`/`expr_c`, Prophet, tape SGD, тензоры (включая `load_ppm`/`load_wav`). Сам гоняет `examples/prophet.kenga` и `mlp_autograd.kenga`.  
Kenga сама пишет C99:

```bat
bootstrap\bin\kenga-lite.exe run kenga\emit\lower_c.kenga
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_kval.kenga
bootstrap\bin\kenga-lite.exe run kenga\emit\lower_kv.kenga
bootstrap\bin\kenga-lite.exe run kenga\emit\bc_src_c.kenga
scripts\freedom-smoke.cmd
```

- `lower_c` — i64/struct/float/agent, `typeof`/`to_str`, `print`/`sleep_ms`/`now_ms` → `bootstrap/bin/lower_*.c`  
- `lower_kv` — tagged KVal + Memory + Tensor/tape как lite (CE: `ag_softmax`/`ag_log`) → `bootstrap/generated/`  
- `bc_src_c` — парсит `.kenga` → bytecode → native VM (`&&` `||` `!` `%`, `typeof`/`to_str`, `print`/`sleep_ms`/`now_ms`, birth/net/agent, **opcodes 106–109** `argc`/`arg`/`file_exists`/`read_line`) → `bootstrap/generated/bc_one_out.c`  
- `rt_*` — Kenga пишет весь lite host (типы, Prophet, tensor, tape, compiler, VM, selftest). `kenga_lite.c` — `#include`.

Это путь, которым `more.kenga` перестанет нуждаться в C-VM host.

## 8. Своя языковая модель

Не скачанный GGUF. Сеть и корпус на Kenga.

```bat
scripts\kenga-birth.cmd
scripts\bc-run.cmd examples\ml\kenga_birth.kenga
bootstrap\bin\kenga-lite.exe run examples\ml\kenga_trigram.kenga
bootstrap\bin\kenga-lite.exe run examples\ml\kenga_mm_lm.kenga
bootstrap\bin\kenga-lite.exe run examples\ml\kenga_mm_gen.kenga
bootstrap\bin\kenga-lite.exe run examples\ml\kenga_mm_words.kenga
scripts\kenga-mm.cmd
```

Birth с промпта `"fn add"` пишет `examples/ml/kenga_born.kenga`, запуск — **24**. Тот же birth собирается в native C (`bc-run`).

Живой Пророк: `examples/ml/living_prophet.kenga`. Спека 400M (Z-ёмкость + смесь проверенных программ): `examples/ml/prophet_400m.kenga`.

Telegram-бот на том же языке: `examples/telegram_bot.kenga` (токен в `TELEGRAM_BOT_TOKEN`). Слышит чат, отвечает из `minds/tg_memory.txt`, тихо учит 16-d Пророка (`minds/tg_prophet.km`). `remember: факт`, `search: запрос`, вопрос с `?`. VPS: `docs/VPS.md`.

## 9. Упражнения

`docs/EXERCISES.md` · `examples/exercises/e01_sum.kenga`

## Дальше

- Книга (PDF/EPUB): `book/kenga_kniga_yantaras_v1.pdf`
- Справочник: `docs/LANGUAGE.md`
- Self-host: `docs/SELFHOST.md`
- Свобода от Rust: `docs/INDEPENDENCE.md`
- Тур: `docs/TOUR.md`
