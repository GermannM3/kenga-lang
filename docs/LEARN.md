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
- `&&` `||` `!` `%` `typeof` `to_str` (lite, `more`, lowerers, `bc_src_c`)
- `print` без `\n`, `sleep_ms` (lite; `lower_c` / `lower_kv` эмитят в C)
- `now_ms` — часы, не заглушка; комментарии `//` и `/* … */`

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

`more.kenga` на своей VM: `examples/selfhost/tensor_more.kenga`, `tape_more.kenga`, `vision_more.kenga`, `examples/prophet.kenga`.

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

- `lower_c` — i64/struct/float/agent, `typeof`/`to_str`, `print`/`sleep_ms` → `bootstrap/bin/lower_*.c`  
- `lower_kv` — tagged KVal + тот же диалект → `bootstrap/generated/`  
- `bc_src_c` — парсит `.kenga` → bytecode → native VM (`&&` `||` `!` `%`, `typeof`/`to_str`, birth/net/agent)  
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
`kenga_mm_gen` — по буквам, стебель `kra`/`ze`/`si`.  
`kenga_mm_words` — цвет одним токеном (`krasnyj` / `zelenyj` / `sinij`) и текст `kenga zhivet v yazyke`. Talk грузит веса.  
Шкала: `docs/KENGA_LM.md`. Куда потом большую: `docs/HUGGINGFACE.md`.

## 9. Упражнения

`docs/EXERCISES.md` · `examples/exercises/e01_sum.kenga`

## Дальше

- Книга (PDF/EPUB): `book/kenga_kniga_yantaras_v1.pdf`
- Справочник: `docs/LANGUAGE.md`
- Self-host: `docs/SELFHOST.md`
- Свобода от Rust: `docs/INDEPENDENCE.md`
- План: `docs/ROADMAP.md`
- Питч: `docs/FOR_FRIENDS.md`
- Своя LM: `docs/KENGA_LM.md`
- Hugging Face: `docs/HUGGINGFACE.md`
