# Путь к самостоятельности (без Rust, потом без C)

Kenga должна **жить на себе**. C и Rust — подмости.

## Сейчас (3.11)

| Слой | Без Rust? | Без C? |
|---|---|---|
| Living runtime | ✅ `kenga-lite` | ❌ C99 host |
| Word-LM / tape / tensor / Prophet | ✅ | ❌ |
| Компилятор на Kenga | ✅ **`kenga/compiler/more.kenga`** (for / elif / struct / import / `xs[i]=`) | ❌ |
| Emit на Kenga | ✅ **`c_seed` + `expr_c`** | ❌ gcc |
| Каталог замены Rust | ✅ **`kenga/`** + `docs/REPLACE_RUST.md` | — |
| Полный CLI `src/` | 🟡 legacy | — |

## Лестница

1. Lite host (C) — living без Rust.  
2. **`kenga/compiler`** вытесняет `src/compiler.rs` + кусок VM.  
3. **`kenga/emit`** вытесняет `codegen.rs`.  
4. Emit полного диалекта → ручной C уходит.  
5. Releases без cargo; `src/` → archive.  
6. VM на Kenga → C уходит.

Карта модулей: **`docs/REPLACE_RUST.md`**.

## Команды

```bat
bootstrap\build.cmd
scripts\freedom-smoke.cmd
bootstrap\bin\kenga-lite.exe run kenga\compiler\more.kenga
bootstrap\bin\kenga-lite.exe run kenga\emit\expr_c.kenga
bootstrap\bin\kenga-lite.exe run examples\ml\word_lm.kenga
```

## Честно

Большая LLM / GPU — позже. Сейчас: **компилятор и LM на том же языке**, что память.  
Rust в `src/` ещё собирает Releases — но новый код туда не кладём.
