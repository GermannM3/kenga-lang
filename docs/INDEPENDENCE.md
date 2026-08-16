# Путь к самостоятельности (без Rust, потом без C)

Kenga должна **жить на себе**. C и Rust — подмости.

## Сейчас (3.13)

| Слой | Без Rust? | Без C? |
|---|---|---|
| Living runtime | ✅ `kenga-lite` целиком из `rt_*.kenga` (типы, Prophet, tensor, tape, compiler, VM) | ❌ C99 glue / gcc |
| Word-LM / tape / tensor / Prophet | ✅ | ❌ |
| Компилятор на Kenga | ✅ **`more.kenga`**: birth→24, XOR, **`c_seed`/`expr_c`**, Prophet, tape, ppm/wav | ❌ |
| Emit на Kenga | ✅ **`lower_c`** + **`lower_kv`/`rt_kval`** | ❌ gcc/cl |
| Native из `.kenga` | ✅ agent / net / **birth** (`bc_from_birth.c` → пишет `kenga_born.kenga`) | ❌ |
| Каталог замены Rust | ✅ **`kenga/`** + `docs/REPLACE_RUST.md` | — |
| Полный CLI `src/` | 🟡 legacy | — |

## Лестница

1. Lite host (C) — living без Rust.  
2. **`kenga/compiler`** вытесняет `src/compiler.rs` + кусок VM.  
3. **`lower_c` / `lower_kv`** вытесняют `codegen.rs`.  
4. **`bc_src_c`**: parse `.kenga` → bytecode → generated C VM. Сетка: **`kenga_net.kenga`**. Birth: **`kenga_birth.kenga`** → native C пишет программу. Запуск одного файла: **`scripts\bc-run.cmd`**.  
5. Emit lite runtime закрыт: `kenga_lite.c` только includes. Дальше — VM на Kenga без C.  
6. Releases: `kenga-lite` обязателен в CI/zip; cargo `kenga` ещё legacy. `src/` → archive, когда zip = только lite.  
7. VM на Kenga → C уходит.

Карта модулей: **`docs/REPLACE_RUST.md`**.

## Команды

```bat
bootstrap\build.cmd
scripts\freedom-smoke.cmd
scripts\bc-run.cmd examples\ml\kenga_birth.kenga
bootstrap\bin\kenga-lite.exe run kenga\compiler\more.kenga
bootstrap\bin\kenga-lite.exe run kenga\emit\lower_c.kenga
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_kval.kenga
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_cli.kenga
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_mem.kenga
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_host.kenga
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_val.kenga
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_lex.kenga
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_arena.kenga
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_parse.kenga
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_loop.kenga
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_prog.kenga
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_scan.kenga
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_expr.kenga
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_factor.kenga
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_stmt.kenga
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_compile.kenga
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_print.kenga
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_vm.kenga
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_selftest.kenga
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_types.kenga
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_prophet.kenga
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_tensor.kenga
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_tape.kenga
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_events.kenga
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_chat.kenga
bootstrap\bin\kenga-lite.exe run kenga\emit\lower_kv.kenga
```

`lower_kv` пишет в `bootstrap/generated/` (KVal runtime + agent/str/lex native).

## Честно

Большая LLM / GPU — позже. Сейчас: **компилятор и LM на том же языке**, что память.  
Rust в `src/` ещё собирает Releases — но новый код туда не кладём.
