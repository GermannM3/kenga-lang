# Self-host ladder (pure Kenga)

Канон теперь в **`kenga/`** (замена `src/`). Здесь — алиасы через `import`.

| Step | File | What |
|---|---|---|
| 1–7 | `arith` … `bc_fn` | лестница VM |
| 8 | `kenga_lite.kenga` → `kenga/compiler/lite.kenga` | i64 compiler |
| 9–10 | `kenga_more.kenga` → **`kenga/compiler/more.kenga`** | f64 / lists / for / elif / struct / import |
| — | `emit_c_seed.kenga` → `kenga/emit/{c_seed,expr_c}.kenga` | Kenga пишет `.c` |

```bat
bootstrap\bin\kenga-lite.exe run kenga\compiler\more.kenga
scripts\freedom-smoke.cmd
```

Карта вытеснения Rust: `docs/REPLACE_RUST.md`.
