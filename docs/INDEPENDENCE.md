# Путь к самостоятельности (без Rust, потом без C)

Kenga должна **жить на себе**. C и Rust — подмости.

## Сейчас (3.12)

| Слой | Без Rust? | Без C? |
|---|---|---|
| Living runtime | ✅ `kenga-lite` | ❌ C99 host |
| Word-LM / tape / tensor / Prophet | ✅ | ❌ |
| Компилятор на Kenga | ✅ **`kenga/compiler/more.kenga`** (for / elif / struct / import / events) | ❌ |
| Emit на Kenga | ✅ **`lower_c`** + seeds (`expr_c` / `core_c`) | ❌ gcc/cl |
| Native из `.kenga` | ✅ agent / for / lists / struct / elif / float → `bootstrap/bin/lower_*.c` | ❌ |
| Каталог замены Rust | ✅ **`kenga/`** + `docs/REPLACE_RUST.md` | — |
| Полный CLI `src/` | 🟡 legacy | — |

## Лестница

1. Lite host (C) — living без Rust.  
2. **`kenga/compiler`** вытесняет `src/compiler.rs` + кусок VM.  
3. **`kenga/emit/lower_c`** вытесняет `codegen.rs` (уже: control + events + struct + f64).  
4. Emit полного lite runtime → ручной `kenga_lite.c` уходит.  
5. Releases без cargo; `src/` → archive.  
6. VM на Kenga → C уходит.

Карта модулей: **`docs/REPLACE_RUST.md`**.

## Команды

```bat
bootstrap\build.cmd
scripts\freedom-smoke.cmd
bootstrap\bin\kenga-lite.exe run kenga\compiler\more.kenga
bootstrap\bin\kenga-lite.exe run kenga\emit\lower_c.kenga
bootstrap\bin\kenga-lite.exe run examples\ml\word_lm.kenga
```

После `lower_c` можно собрать, например, `bootstrap\bin\lower_agent.exe` — тот же `examples/agent.kenga`, но уже как native C.

## Честно

Большая LLM / GPU — позже. Сейчас: **компилятор и LM на том же языке**, что память.  
Rust в `src/` ещё собирает Releases — но новый код туда не кладём.
