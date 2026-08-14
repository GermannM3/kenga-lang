# Bootstrap 1.3

## Что уже «твоё»

- Программы на `.kenga` (включая нейромодель)
- Обученный mind в `minds/*.km`
- `kenga chat` — русский диалог с world-model
- Self-host seed: арифметика на чистом Kenga

## Что ещё bootstrap

Компилятор/VM хостятся на **Rust**. Python не нужен.

| Слой | Статус |
|---|---|
| Язык + VM | ✅ |
| Prophet residual MLP | ✅ |
| Pure-Kenga neuromodel | ✅ |
| save/load + chat | ✅ |
| Self-host seed (arith) | ✅ |
| Полный self-host | 🚧 |
| LLM-чат «как GPT» | ❌ не эта модель |

См. `docs/SELFHOST.md`.
