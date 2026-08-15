# Self-host ladder (pure Kenga)

Канон теперь в **`kenga/`** (замена `src/`). Здесь — алиасы через `import`.

| Step | File | What |
|---|---|---|
| 1–7 | `arith` … `bc_fn` | лестница VM |
| 8 | `kenga_lite.kenga` → `kenga/compiler/lite.kenga` | i64 compiler |
| 9 | `kenga_more.kenga` → **`kenga/compiler/more.kenga`** | f64 / lists / str / true / ord |
| — | `emit_c_seed.kenga` → `kenga/emit/c_seed.kenga` | Kenga пишет `.c` |

```bat
bootstrap\bin\kenga-lite.exe run kenga\compiler\more.kenga
bootstrap\bin\kenga-lite.exe run examples\selfhost\kenga_more.kenga
```

Карта вытеснения Rust: `docs/REPLACE_RUST.md`.
