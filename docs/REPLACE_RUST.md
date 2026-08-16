# Замена `src/` на `.kenga`

Правило: **новый функционал — сначала в `kenga/` или `bootstrap/`, не в Rust.**  
`src/` только чиним, пока Releases ещё собираются cargo.

| Rust (`src/`) | Статус | Kenga / C |
|---|---|---|
| `lexer.rs` / `token.rs` | 🟡 частично | парсер внутри `kenga/compiler/*.kenga` |
| `parser.rs` / `ast.rs` | 🟡 частично | то же (рекурсивный descent в `.kenga`) |
| `compiler.rs` / `bytecode.rs` | 🟢 растёт | `more.kenga` + `rt_factor`/`rt_stmt`/`rt_compile` пишут lite compiler |
| `vm.rs` (ядро) | 🟢 растёт | VM в `more.kenga` + **`rt_vm.kenga`** пишет C VM lite |
| `tensor.rs` | ✅ на lite | `kenga/emit/rt_tensor.kenga` → `generated/rt_tensor.inc.c` |
| `autograd.rs` | ✅ на lite | `kenga/emit/rt_tape.kenga` |
| `memory.rs` | ✅ на lite | `kenga/emit/rt_prophet.kenga` |
| `talk.rs` (chat) | ✅ на lite | `kenga/emit/rt_chat.kenga` |
| `codegen.rs` (emit-c) | 🟢 растёт | `lower_c` + **`lower_kv`/`rt_kval`** (KVal path к `more`) |
| `main.rs` / `driver.rs` | 🟡 | CLI `main` пишет `kenga/emit/rt_cli.kenga` |
| `demo.rs` / `build.rs` | ⬜ | позже |

## Как добиваем Rust

1. `kenga/compiler/more.kenga` покрывает диалект `*_lite` + всё больше examples.  
2. Emit из Kenga пишет C/bytecode. `kenga_lite.c` — каркас из `#include`.  
3. Releases = только lite-бинарник; `cargo` уходит из README.  
4. `src/` удаляем или архивируем в `legacy/`, когда п.2–3 зелёные.

Проверка сегодня:

```bat
scripts\freedom-smoke.cmd
bootstrap\bin\kenga-lite.exe run kenga\compiler\more.kenga
```

`more` уже гоняет `for_lite` / `elif_lite` / `struct_lite` / `float_lite` / `lists_lite` / **`agent.kenga`** / **`prophet.kenga`** / **`mlp_autograd`** / vision через свой bytecode VM.  
`bc_src_c` пишет тот же диалект в native C (`bc_from_agent`, `bc_from_import`, **`bc_from_net`**, **`bc_from_birth`**, logic, typeof, print).  
`rt_*` пишут весь lite host, включая типы, Prophet, tensor, tape, events, chat, compiler, VM. `kenga_lite.c` — комментарий, CRT includes и `#include`.
