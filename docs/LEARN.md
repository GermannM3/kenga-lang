# Учим Kenga за один вечер

Не по наитию — по шагам. Друзьям проще начать **без Rust**: бинарник с Releases или `bootstrap/`. Полный `cargo install` — если хочешь GPU/legacy path.

```bat
bootstrap\build.cmd
bootstrap\bin\kenga-lite.exe version
```

или с Releases: `kenga version` (3.12+).

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
```

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

Компилятор и VM на Kenga: `kenga/compiler/more.kenga`.  
Kenga сама пишет C99:

```bat
bootstrap\bin\kenga-lite.exe run kenga\emit\lower_c.kenga
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_kval.kenga
bootstrap\bin\kenga-lite.exe run kenga\emit\lower_kv.kenga
scripts\freedom-smoke.cmd
```

- `lower_c` — i64/struct/float/agent → `bootstrap/bin/lower_*.c`  
- `lower_kv` — tagged KVal (str/ord/hetero lists) + **lex_frag** → `bootstrap/generated/`  
- `bc_src_c` — **парсит** `.kenga` → bytecode → native VM (`bc_from_fn`→42, lists→19, fact→120, for_lite, `else if`)

Это путь, которым `more.kenga` перестанет нуждаться в C-VM host.

## 8. Упражнения

`docs/EXERCISES.md` · `examples/exercises/e01_sum.kenga`

Своя языковая модель (decoder, не XOR): `examples/ml/kenga_lm.kenga` — зачем и какая шкала: `docs/KENGA_LM.md`.

## Дальше

- Справочник: `docs/LANGUAGE.md`
- Self-host: `docs/SELFHOST.md`
- Свобода от Rust: `docs/INDEPENDENCE.md`
- План: `docs/ROADMAP.md`
- Питч: `docs/FOR_FRIENDS.md`
