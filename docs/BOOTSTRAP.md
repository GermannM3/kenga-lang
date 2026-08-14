# Bootstrap 1.2

## Что уже «твоё»

- Программы на `.kenga` (включая нейромодель)
- Обученный mind в `minds/*.km` — сохраняется и грузится
- `kenga talk` — интерактив с world-model

## Что ещё bootstrap

Компилятор и VM пока хостятся на **Rust**. Это нормальный этап языка (как Go на C).  
**Python не нужен.** Self-host — следующий крупный шаг.

| Слой | Статус |
|---|---|
| Язык + VM | ✅ |
| Prophet residual MLP | ✅ |
| Pure-Kenga neuromodel | ✅ |
| save/load mind + talk | ✅ |
| emit-c / build | ✅ |
| Self-host компилятор | 🚧 |
| LLM-чат «как GPT» | ❌ не цель этой модели |

World-model предсказывает динамику состояний (`[pos,vel,fuel]→next`), а не пишет стихи.
