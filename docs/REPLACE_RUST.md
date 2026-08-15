# Замена `src/` на `.kenga`

Правило: **новый функционал — сначала в `kenga/` или `bootstrap/`, не в Rust.**  
`src/` только чиним, пока Releases ещё собираются cargo.

| Rust (`src/`) | Статус | Kenga / C |
|---|---|---|
| `lexer.rs` / `token.rs` | 🟡 частично | парсер внутри `kenga/compiler/*.kenga` |
| `parser.rs` / `ast.rs` | 🟡 частично | то же (рекурсивный descent в `.kenga`) |
| `compiler.rs` / `bytecode.rs` | 🟡 растёт | `kenga/compiler/more.kenga` |
| `vm.rs` (ядро) | 🟡 растёт | VM в `more.kenga` + полный runtime в `bootstrap/` |
| `tensor.rs` | ✅ на lite | `bootstrap/tensor_lite.inc.c` |
| `autograd.rs` | ✅ на lite | `bootstrap/tape_lite.inc.c` |
| `memory.rs` | ✅ на lite | `bootstrap/prophet_lite.inc.c` |
| `talk.rs` (chat) | ✅ на lite | `bootstrap/chat_lite.inc.c` |
| `codegen.rs` (emit-c) | 🟡 зерно | `kenga/emit/c_seed.kenga` |
| `main.rs` / `driver.rs` | ⬜ | CLI = `kenga-lite` пока |
| `demo.rs` / `build.rs` | ⬜ | позже |

## Как добиваем Rust

1. `kenga/compiler/more.kenga` покрывает диалект `*_lite` + всё больше examples.  
2. Emit из Kenga пишет C/bytecode → `bootstrap/kenga_lite.c` перестаёт правиться руками.  
3. Releases = только lite-бинарник; `cargo` уходит из README.  
4. `src/` удаляем или архивируем в `legacy/`, когда п.2–3 зелёные.

Проверка сегодня:

```bat
bootstrap\bin\kenga-lite.exe run kenga\compiler\more.kenga
```
