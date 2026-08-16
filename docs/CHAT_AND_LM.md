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
| **Decoder GPT-формы** | ✅ `examples/ml/kenga_lm.kenga` — attn + FFN + RMS, next-token |
| **Char-LM на нашем `.kenga`** | ✅ `examples/ml/kenga_charlm.kenga` + `kenga_char_talk.kenga` |
| **Триграмма на нашем `.kenga`** | ✅ `examples/ml/kenga_trigram.kenga` — list/i64, `fn add(a: i64` |
| **Birth → run** | ✅ lite и **native C** (`bc-run` / `bc_from_birth.c`) → **24** |
| **Vision+audio → text** | ✅ linear + char-stem + **word-decoder** (`kenga_mm_words` → полная подпись) |
| Hugging Face | ⬜ большая модель → `Kenga-ai/kenga-mm`; сид: `docs/HUGGINGFACE.md` |
| Половина Grok / GPU | ⬜ те же блоки × D/L/V + корпус + GPU; см. `docs/KENGA_LM.md` |

```bat
bootstrap\bin\kenga-lite.exe run examples\ml\word_lm.kenga
bootstrap\bin\kenga-lite.exe run examples\ml\tiny_lm.kenga
bootstrap\bin\kenga-lite.exe run examples\ml\kenga_charlm.kenga
bootstrap\bin\kenga-lite.exe run examples\ml\kenga_trigram.kenga
scripts\kenga-birth.cmd
scripts\bc-run.cmd examples\ml\kenga_birth.kenga
bootstrap\bin\kenga-lite.exe run examples\ml\kenga_mm_gen.kenga
bootstrap\bin\kenga-lite.exe run examples\ml\kenga_mm_words.kenga
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
