# `kenga/` — замена Rust-хоста

Здесь растёт **нативный** код Kenga, который со временем вытеснит `src/*.rs`.

C99 `bootstrap/` — временный runtime. Rust `src/` — legacy-хост для Releases, пока лестница не закроется.

```bat
bootstrap\bin\kenga-lite.exe run kenga\compiler\more.kenga
bootstrap\bin\kenga-lite.exe run kenga\emit\c_seed.kenga
bootstrap\bin\kenga-lite.exe run kenga\emit\expr_c.kenga
scripts\freedom-smoke.cmd
```

| Путь | Роль | Вместо Rust |
|---|---|---|
| `compiler/lite.kenga` | i64 compiler+VM на Kenga | кусок `compiler.rs` + `vm.rs` |
| `compiler/more.kenga` | f64 / lists / str / for / elif / struct / import | шире `compiler.rs` + `vm.rs` |
| `emit/c_seed.kenga` | Kenga пишет `.c` | зародыш `codegen.rs` |
| `emit/expr_c.kenga` | expr → C99 + self-check | следующий шаг `codegen.rs` |
| `emit/control_c.kenga` | while/if factorial → C99 | control в emit |

Статус по модулям: `docs/REPLACE_RUST.md`.
