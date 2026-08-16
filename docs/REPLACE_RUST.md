# Замена `src/` на `.kenga`

Правило: **новый функционал — сначала в `kenga/` или `bootstrap/`, не в Rust.**  
`src/` только чиним, пока Releases ещё собираются cargo.

| Rust (`src/`) | Статус | Kenga / C |
|---|---|---|
| `lexer.rs` / `token.rs` | 🟡 частично | парсер внутри `kenga/compiler/*.kenga` |
| `parser.rs` / `ast.rs` | 🟡 частично | то же (рекурсивный descent в `.kenga`) |
| `compiler.rs` / `bytecode.rs` | 🟢 растёт | `kenga/compiler/more.kenga` (for/elif/struct/import/SET/events) |
| `vm.rs` (ядро) | 🟢 растёт | VM в `more.kenga` (вкл. emit/pump) + runtime в `bootstrap/` |
| `tensor.rs` | ✅ на lite | `bootstrap/tensor_lite.inc.c` |
| `autograd.rs` | ✅ на lite | `bootstrap/tape_lite.inc.c` |
| `memory.rs` | ✅ на lite | `bootstrap/prophet_lite.inc.c` |
| `talk.rs` (chat) | ✅ на lite | `bootstrap/chat_lite.inc.c` |
| `codegen.rs` (emit-c) | 🟢 растёт | `lower_c` + **`lower_kv`/`rt_kval`** (KVal path к `more`) |
| `main.rs` / `driver.rs` | 🟡 | CLI `main` пишет `kenga/emit/rt_cli.kenga` |
| `demo.rs` / `build.rs` | ⬜ | позже |

## Как добиваем Rust

1. `kenga/compiler/more.kenga` покрывает диалект `*_lite` + всё больше examples.  
2. Emit из Kenga пишет C/bytecode → `bootstrap/kenga_lite.c` перестаёт правиться руками.  
3. Releases = только lite-бинарник; `cargo` уходит из README.  
4. `src/` удаляем или архивируем в `legacy/`, когда п.2–3 зелёные.

Проверка сегодня:

```bat
scripts\freedom-smoke.cmd
bootstrap\bin\kenga-lite.exe run kenga\compiler\more.kenga
```

`more` уже гоняет `for_lite` / `elif_lite` / `struct_lite` / `float_lite` / `lists_lite` / **`agent.kenga`** через свой bytecode VM.  
`bc_src_c` пишет тот же диалект в native C (`bc_from_agent`, `bc_from_import`, **`bc_from_net`**, **`bc_from_birth`**).  
`rt_cli` / `rt_mem` / `rt_host` пишут `main`, die/xstrdup и import-loader.
