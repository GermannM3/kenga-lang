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
| `compiler/more.kenga` | f64 / lists / str / for / elif / struct / import / events | шире `compiler.rs` + `vm.rs` |
| `emit/c_seed.kenga` | Kenga пишет `.c` | зародыш `codegen.rs` |
| `emit/expr_c.kenga` | expr → C99 + self-check | следующий шаг `codegen.rs` |
| `emit/mini_codegen.kenga` | alias → `core_c` | — |
| `emit/core_c.kenga` | parse while/if/for/list → C99 | multi-stmt templates |
| `emit/lower_c.kenga` | recursive descent → C99 (fn/str/list) | вытесняет кусок `codegen.rs` |

Статус по модулям: `docs/REPLACE_RUST.md`.
