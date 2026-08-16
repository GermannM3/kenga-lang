# Tensor на kenga-lite (без Rust)

С **3.3** плотные f64-тензоры пишет `kenga/emit/rt_tensor.kenga` → `bootstrap/generated/rt_tensor.inc.c`.

```bat
bootstrap\bin\kenga-lite.exe run examples\ml\tensor_core.kenga
kenga run --lite examples\ml\tensor_core.kenga
```

## API

| Вызов | Смысл |
|---|---|
| `tensor(d0, d1, …)` | нули, shape из dims |
| `t_from([…], […])` | shape + flat data |
| `t_fill` / `t_get` / `t_set` / `t_shape` | доступ |
| `t_add` / `t_sub` / `t_mul` | elementwise |
| `t_matmul` | rank-2 |
| `t_dot` / `t_sum` / `t_scale` | → f64 / scale |
| `t_reshape` / `t_transpose` | форма |
| `t_exp` / `t_log` / `t_softmax` | поэлементно / log(max(x,eps)) / вектор |
| `t_mean` | rank-3 → `[c]`; иначе f64 / `[1]` |
| `t_mse(a, b)` | mean squared error → f64 |
| `t_patch_mean(t, gh, gw)` | `[h,w,c]` → `[gh,gw,c]` mean-pool |
| `t_linear_grad(w, x, y)` | grad `W` для `mse(W@x, y)` |
| `save_tensor` / `load_tensor` | текст `KENGA_TENSOR 1` |
| `write_file` / `read_file` | строки на диск |
| `load_ppm("…")` | P6 → `[h,w,3]` в 0..1 |
| `load_wav("…")` | PCM16 → `[n]` в -1..1 |
| `sweep()` | no-op → 0 |

Living без Rust:

```bat
kenga run --lite examples\ml\living_multimodal.kenga
kenga chat --lite minds\multi.km
kenga run --lite examples\ml\autograd_tape.kenga
```

Tape: `docs/TAPE_LITE.md`. Пока не на lite: GPU.
