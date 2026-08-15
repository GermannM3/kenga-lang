# Учим Kenga за один вечер

Не по наитию — по шагам. Нужен клон репо и Rust (пока полный `kenga` на нём).

```bash
cargo install --path . --force --locked
kenga version   # 2.3.x
```

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
kenga run examples/hello.kenga
```

## 2. Условия и циклы

Смотри `examples/showcase.kenga`. Идеи:

- `if cond { … } else { … }`
- `while cond { … }`
- `for i in 0..n { … }` / `for x in xs { … }`

## 3. Списки и struct

`examples/native_lists.kenga`, `examples/native_struct.kenga`:

```kenga
let xs = [1, 2, 3];
xs = push(xs, 4);
println(len(xs));
```

## 4. События агента

`examples/agent.kenga` — `on` / `emit` / `pump`.

## 5. Память и world-model

```bash
kenga run examples/ml/world_model.kenga
kenga run examples/ml/surprise_gate.kenga
kenga chat minds/agent.km
```

API: `memory`, `learn`, `predict`, `unroll`, `surprise`, `consolidate` — см. `docs/LANGUAGE.md`.

## 6. Тензоры и multimodal bridges

```bash
kenga run examples/ml/tensor_core.kenga
kenga run examples/ml/train_sgd.kenga
kenga run examples/ml/vision_ppm.kenga
kenga run examples/ml/fusion.kenga
```

## 7. Без Rust (lite)

```bat
bootstrap\build.cmd
kenga run --lite examples\selfhost\hello_lite.kenga
kenga run --lite examples\selfhost\lists_lite.kenga
kenga run --lite examples\selfhost\struct_lite.kenga
kenga run --lite examples\selfhost\float_lite.kenga
kenga run --lite examples\selfhost\elif_lite.kenga
kenga run --lite examples\selfhost\fact_lite.kenga
```

Lite: `fn`/`let`/`while`/`if`/`return`, строки, i64-списки, `struct`, `println`.

## 8. Упражнения

`docs/EXERCISES.md` · `examples/exercises/e01_sum.kenga`

## Дальше

- Справочник: `docs/LANGUAGE.md`
- Self-host: `docs/SELFHOST.md`
- План: `docs/ROADMAP.md`
- Питч: `docs/FOR_FRIENDS.md`
