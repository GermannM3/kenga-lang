# Chat, tiny LM и свобода от Rust/C

## Честно про «умную как Grok»

`kenga chat` + `minds/multi.km` — живой world-model (векторы), не чат-LLM.  
Текст растёт отдельно — на том же языке и tape.

| Слой | Статус |
|---|---|
| Prophet / multimodal | ✅ lite без Rust |
| Chat intents | ✅ `kenga-lite chat` |
| Bigram LM | ✅ `examples/ml/tiny_lm.kenga` |
| **2-layer word-LM + CE** | ✅ `examples/ml/word_lm.kenga` → `minds/word_lm_*.kt` |
| **Своя MLP (XOR)** | ✅ `examples/ml/kenga_net.kenga` — list/f64, без tensor host |
| Большая LLM / GPU | ⬜ данные + слои + (позже) GPU |

```bat
bootstrap\bin\kenga-lite.exe run examples\ml\word_lm.kenga
bootstrap\bin\kenga-lite.exe run examples\ml\tiny_lm.kenga
kenga chat --lite minds\multi.km
```

## Свобода от Rust и C

Актуальная карта: **`docs/INDEPENDENCE.md`**.

| Есть без Rust | Ещё нужно |
|---|---|
| Весь living runtime на `kenga-lite` (C99) | Убрать ручной C → codegen из `.kenga` |
| Word-LM / tape / tensor / Prophet / events | Self-host VM на Kenga |
| `emit_c_seed.kenga` пишет `.c` через `write_file` | Полный emit диалекта из Kenga |
| Лестница `examples/selfhost/` | Полный CLI без `src/` |

Итог: **друзьям Rust не нужен**. C — временный host. Цель — язык, который компилирует и учит себя сам.
