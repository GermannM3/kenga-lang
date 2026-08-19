# Замена `src/` на `.kenga`

Правило: **новый функционал — сначала в `kenga/` или `bootstrap/`, не в Rust.**  
`src/` только чиним, пока Releases ещё собираются cargo.

| Rust (`src/`) | Статус | Kenga / C |
|---|---|---|
| `lexer.rs` / `token.rs` | 🟡 частично | парсер внутри `kenga/compiler/*.kenga` |
| `parser.rs` / `ast.rs` | 🟡 частично | то же (рекурсивный descent в `.kenga`) |
| `compiler.rs` / `bytecode.rs` | 🟢 растёт | `more.kenga` + `rt_factor`/`rt_stmt`/`rt_compile` пишут lite compiler |
| `vm.rs` (ядро) | 🟢 растёт | VM в `more.kenga` (fast: sp-стек + hoisted dispatch) + **`rt_vm.kenga`** пишет C VM lite |
| `tensor.rs` | ✅ на lite | `kenga/emit/rt_tensor.kenga` → `generated/rt_tensor.inc.c` |
| `autograd.rs` | ✅ на lite + KVal emit | `rt_tape.kenga` / `rt_kval_tape.kenga` |
| `memory.rs` | ✅ на lite | `kenga/emit/rt_prophet.kenga` |
| `talk.rs` (chat) | ✅ на lite | `kenga/compiler/chat.kenga` + `ml_host`; C chat не в `kenga_lite.c` |
| `codegen.rs` (emit-c) | 🟢 растёт | `lower_c` + **`lower_kv`**: Tensor/tape как lite, native CE |
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

`kenga-lite run file.kenga` гоняет файл на **more VM**. Сьют `more.kenga` без аргументов. `kenga/emit/*` пишет host и остаётся lite bootstrap.  
`bc_src_c` пишет тот же диалект в native C (`bc_from_agent`, `bc_from_import`, **`bc_from_net`**, **`bc_from_birth`**, logic, typeof, print, **`bc_from_argc`** — opcodes 106–109).  
`rt_*` пишут весь lite host, включая типы, Prophet, tensor, tape, events, chat, compiler, VM. `kenga_lite.c` — комментарий, CRT includes и `#include`.

## Полный bootstrap без Rust

`bc_src_c` закрывает путь «.kenga → native exe»:

```
# 1. Kenga-исходник генерирует bc_rt.inc.c + bc_from_*.c:
bootstrap/bin/kenga-lite.exe run kenga/emit/bc_src_c.kenga
#   → bootstrap/generated/bc_rt.inc.c (runtime с OP_ARGC/ARG/FILE_EXISTS/READ_LINE)
#   → bootstrap/generated/bc_from_agent.c, bc_from_argc.c, bc_one_out.c, …

# 2. Любой штатный C-компилятор (msvc / gcc / clang) собирает native exe:
cl  /O2 /TC bc_one_out.c /Febc_one_out.exe        :: MSVC
gcc -O2 -std=c99 bc_one_out.c -o bc_one_out.exe   ;; MinGW / Linux
```

`bc_src_c` теперь умеет пробрасывать host-`argc`/`argv` в VM через `g_kargc`/`g_kargv` (opcodes 106–109). Любая `.kenga` программа корректно читает `argc()`, `arg(i)`, `file_exists(p)`, `read_line()` — нативно, без интерпретатора.
