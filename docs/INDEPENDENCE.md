# Путь к самостоятельности (без Rust, потом без C)

Kenga должна **жить на себе**. C и Rust — подмости.

## Сейчас (3.12)

| Слой | Без Rust? | Без C? |
|---|---|---|
| Living runtime | ✅ `kenga-lite` | ❌ C99 host |
| Word-LM / tape / tensor / Prophet | ✅ | ❌ |
| Компилятор на Kenga | ✅ **`kenga/compiler/more.kenga`** (for / elif / struct / import / events) | ❌ |
| Emit на Kenga | ✅ **`lower_c`** + **`lower_kv`/`rt_kval`** | ❌ gcc/cl |
| Native из `.kenga` | ✅ agent / struct / float + **str/ord/lex_frag** (KVal) | ❌ |
| Каталог замены Rust | ✅ **`kenga/`** + `docs/REPLACE_RUST.md` | — |
| Полный CLI `src/` | 🟡 legacy | — |

## Лестница

1. Lite host (C) — living без Rust.  
2. **`kenga/compiler`** вытесняет `src/compiler.rs` + кусок VM.  
3. **`lower_c` / `lower_kv`** вытесняют `codegen.rs`.  
4. **`bc_src_c`**: parse `.kenga` → bytecode → generated C VM (lite + agent + import + file I/O + **V-lists**). Своя сетка: **`examples/ml/kenga_net.kenga`**. Decoder на нашем исходнике: **`examples/ml/kenga_charlm.kenga`**. Запуск одного файла: **`scripts\bc-run.cmd`**.  
5. Emit полного lite runtime (`rt_kval` → host) → ручной `kenga_lite.c` уходит.  
6. Releases без cargo; `src/` → archive.  
7. VM на Kenga → C уходит.

Карта модулей: **`docs/REPLACE_RUST.md`**.

## Команды

```bat
bootstrap\build.cmd
scripts\freedom-smoke.cmd
scripts\bc-run.cmd examples\selfhost\fact_lite.kenga
bootstrap\bin\kenga-lite.exe run kenga\compiler\more.kenga
bootstrap\bin\kenga-lite.exe run kenga\emit\lower_c.kenga
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_kval.kenga
bootstrap\bin\kenga-lite.exe run kenga\emit\lower_kv.kenga
```

`lower_kv` пишет в `bootstrap/generated/` (KVal runtime + agent/str/lex native).

## Честно

Большая LLM / GPU — позже. Сейчас: **компилятор и LM на том же языке**, что память.  
Rust в `src/` ещё собирает Releases — но новый код туда не кладём.
