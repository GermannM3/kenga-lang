# Prophet Memory на kenga-lite (без Rust)

С **3.2** ядро Prophet + `.km` + chat живут в lite без Rust. C пишет `kenga/emit/rt_prophet.kenga` и `rt_chat.kenga`. Демо `examples/prophet.kenga` и `minds/multi.km`.

```bat
bootstrap\build.cmd
bootstrap\bin\kenga-lite.exe run examples\prophet.kenga
```

```bash
bash bootstrap/build.sh
./bootstrap/bin/kenga-lite run examples/prophet.kenga
# или:
kenga run --lite examples/prophet.kenga
```

## API на lite

| Вызов | Смысл |
|---|---|
| `memory_config(thr, ep_cap, core_cap)` | thr: i64 `10` → `0.10`, либо f64 |
| `remember(mind, pat, surprise)` | surprise: i64 `/100` или f64 |
| `surprise(a, b)` | RMSE паттернов → f64 |
| `foresee(mind, obs)` | world-model / core → list f64 |
| `consolidate(mind)` | sleep: episodic → core + EWC-lite locks |
| `mem_stats(mind)` | `[ep, core, locked, steps, dim, hidden]` |
| `recall(mind, query, k)` | top-k похожих паттернов |
| `save_mind(mind, "path.km")` | KENGA_MIND 1 (совместим с Rust) |
| `load_mind("path.km")` | → Memory |

World-model: residual MLP `y ≈ x + Δ`, tanh hidden, CPU f64.

## Chat без Rust

```bat
bootstrap\bin\kenga-lite.exe chat minds\multi.km
kenga chat --lite minds\multi.km
bootstrap\bin\kenga-lite.exe chat minds\multi.km --script examples\ml\chat_script_lite.txt
```

Пока **не** на lite: Tensor ops, events, `ag_*`.

