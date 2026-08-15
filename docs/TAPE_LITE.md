# Tape autograd на kenga-lite (без Rust)

С **3.6** reverse-mode tape живёт в `bootstrap/tape_lite.inc.c`.

```bat
bootstrap\bin\kenga-lite.exe run examples\ml\autograd_tape.kenga
bootstrap\bin\kenga-lite.exe run examples\ml\mlp_autograd.kenga
kenga run --lite examples\ml\autograd_tape.kenga
```

## API

| Вызов | Смысл |
|---|---|
| `ag_clear()` | очистить tape |
| `ag_param(t)` | leaf с `requires_grad` → node id |
| `ag_const(t\|num)` | leaf без градиента |
| `ag_add` / `ag_sub` / `ag_mul` | elementwise |
| `ag_matmul` | rank-2 |
| `ag_scale(id, s)` | × scalar |
| `ag_relu` / `ag_neg` | активации |
| `ag_transpose` / `ag_reshape` | форма |
| `ag_exp` / `ag_log` / `ag_softmax` | поэлементно / log / вектор |
| `ag_mse(pred, target)` | scalar MSE loss |
| `ag_sum(id)` | reduce → scalar node |
| `ag_backward(loss)` | reverse pass |
| `ag_grad(id)` / `ag_value(id)` | Tensor \| f64 |
| `ag_step(param, lr)` | `value - lr * grad` → Tensor |

Демо `autograd_tape.kenga`: учит `W` для `y ≈ W @ x`.  
LM: `examples/ml/word_lm.kenga` (2-layer + CE через `ag_log`).

Пока не на lite: GPU. Путь без C: `docs/INDEPENDENCE.md`.
