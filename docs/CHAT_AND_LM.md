# Chat, tiny LM и свобода от Rust/C

## Честно про «умную как Grok»

`kenga chat` + `minds/multi.km` — живой world-model (векторы), не чат-LLM.  
Текст растёт отдельно — на том же языке и tape.

| Слой | Статус |
|---|---|
| Prophet / multimodal | ✅ lite без Rust |
| Chat intents | ✅ `kenga-lite chat` → `chat.kenga` / `native_ml` |
| Bigram LM | ✅ `examples/ml/tiny_lm.kenga` |
| **2-layer word-LM + CE** | ✅ `examples/ml/word_lm.kenga` (~6 с) и `word_lm_big.kenga` (V=20/H=16, ~26 с) → `minds/word_lm_*.kt`; lite и `more` VM |
| **Своя MLP (XOR)** | ✅ `examples/ml/kenga_net.kenga` — list/f64, без tensor host; lite, bc→C и `more` VM |
| **Decoder GPT-формы** | ✅ `examples/ml/kenga_lm.kenga` — attn + FFN + RMS, next-token |
| **Char-LM на нашем `.kenga`** | ✅ `examples/ml/kenga_charlm.kenga` + `kenga_char_talk.kenga` |
| **Триграмма на нашем `.kenga`** | ✅ `examples/ml/kenga_trigram.kenga` — list/i64, `fn add(a: i64` |
| **Birth → run** | ✅ lite, native C, **`more.kenga` пишет и запускает** `kenga_born.kenga` → **24** |
| **Vision+audio → text** | ✅ word-decoder: 3 подписи + `kenga zhivet v yazyke` (12 токенов, CPU) |
| Hugging Face | ⬜ большая модель → `Kenga-ai/kenga-mm`; сид: `docs/HUGGINGFACE.md` |
| Половина Grok / GPU | ⬜ те же блоки × D/L/V + корпус + GPU; см. `docs/KENGA_LM.md` |

```bat
bootstrap\bin\kenga-lite.exe run examples\ml\word_lm.kenga
scripts\bc-run.cmd examples\ml\word_lm_big.kenga
```
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
| Living runtime целиком из `rt_*.kenga` | C glue + Releases ещё `src/` |
| Word-LM / tape / tensor / Prophet / events | Self-host VM без C glue |
| `emit_c_seed.kenga` пишет `.c` через `write_file` | Archive `src/` когда Releases = lite-only |
| Лестница `examples/selfhost/` | GPU + большой корпус |

Итог: **друзьям Rust не нужен**. C — временный host. Цель — язык, который компилирует и учит себя сам.
