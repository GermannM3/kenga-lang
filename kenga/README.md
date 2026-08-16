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
| `compiler/more.kenga` | birth→24, vision, `print`/`learn`/`unroll`/`foresee_n` | шире `compiler.rs` + `vm.rs` |
| `emit/c_seed.kenga` | Kenga пишет `.c` | зародыш `codegen.rs` |
| `emit/expr_c.kenga` | expr → C99 + self-check | следующий шаг `codegen.rs` |
| `emit/mini_codegen.kenga` | alias → `core_c` | — |
| `emit/core_c.kenga` | parse while/if/for/list → C99 | multi-stmt templates |
| `emit/lower_c.kenga` | recursive → C99 (events/struct/f64) | кусок `codegen.rs` |
| `emit/rt_kval.kenga` | tagged KVal + events + file IO runtime | зародыш runtime host |
| `emit/rt_kval_mem.kenga` | Prophet world-model на KVal | кусок `memory.rs` в native emit |
| `emit/rt_kval_tensor.kenga` | dense f64 tensors на KVal | кусок `tensor.rs` в native emit |
| `emit/rt_kval_tape.kenga` | reverse-mode tape на KVal | кусок `autograd.rs` в native emit |
| `emit/rt_cli.kenga` | `main` kenga-lite → `generated/rt_cli.inc.c` | кусок `main.rs` / ручного host |
| `emit/rt_mem.kenga` | die / xrealloc / xstrdup | кусок runtime host |
| `emit/rt_host.kenga` | read_file + import flatten | кусок driver.rs |
| `emit/rt_val.kenga` | V_i64 / V_f64 / to_f64 | кусок vm.rs |
| `emit/rt_lex.kenga` | intern / skip / starts_kw | кусок lexer.rs |
| `emit/rt_arena.kenga` | i64a/vala/stra/list/struct heaps | кусок runtime host |
| `emit/rt_parse.kenga` | ident / number / string / emit2 | кусок lexer.rs + compiler.rs |
| `emit/rt_loop.kenga` | break/continue + CALL patches | кусок compiler.rs |
| `emit/rt_prog.kenga` | program_free | кусок runtime host |
| `emit/rt_scan.kenga` | type annot / slice / braces | кусок parser.rs |
| `emit/rt_expr.kenga` | * / + - compare / `{ }` | кусок compiler.rs |
| `emit/rt_factor.kenga` | literals / calls / postfix | кусок compiler.rs |
| `emit/rt_stmt.kenga` | let / if / for / println | кусок compiler.rs |
| `emit/rt_compile.kenga` | compile_lite | кусок compiler.rs |
| `emit/rt_print.kenga` | print_value | кусок vm.rs |
| `emit/rt_vm.kenga` | bytecode VM | кусок vm.rs |
| `emit/rt_selftest.kenga` | run_lite + 70 cases | кусок demo.rs |
| `emit/rt_types.kenga` | opcodes + Value/Program | кусок bytecode.rs |
| `emit/rt_prophet.kenga` | Prophet Memory | `memory.rs` |
| `emit/rt_tensor.kenga` | dense f64 tensors | `tensor.rs` |
| `emit/rt_tape.kenga` | reverse-mode tape | `autograd.rs` |
| `emit/rt_events.kenga` | on/emit/pump | events |
| `emit/rt_chat.kenga` | chat intents | `talk.rs` |
| `emit/lower_kv.kenga` | KVal lowerer (str/ord/hetero/lex) | путь к native `more` |

Статус по модулям: `docs/REPLACE_RUST.md`.

Свои сети живут в `examples/ml/` (birth в т.ч. native C, char-LM, PPM+WAV→текст). Учебник: `docs/LEARN.md` §8. HF: `docs/HUGGINGFACE.md`.
