# Своя языковая модель на Kenga

Половина Grok — это не другой алгоритм. Это **тот же decoder** (внимание, residual, FFN, норма, LM-head) с другими числами: ширина, глубина, словарь, данные, GPU.

Kenga уже выражает эту машину. Файлы:

- `examples/ml/kenga_lm.kenga` — decoder на закрытом словесном словаре
- `examples/ml/kenga_charlm.kenga` — тот же decoder, корпус = наши `.kenga`
- `examples/ml/kenga_trigram.kenga` — char-триграмма на list/i64 (без Tensor), тот же корпус
- `examples/ml/kenga_birth.kenga` — suffix LM пишет `kenga_born.kenga`, lite его запускает → **24**
- `examples/ml/kenga_mm_lm.kenga` — PPM+WAV → подпись (linear)
- `examples/ml/kenga_mm_gen.kenga` — decoder пишет стебель цвета (kra/ze/si)
- `examples/ml/kenga_mm_words.kenga` — тот же decoder, цвет = один токен, полная подпись

Скачанный GGUF из папки «kenga ai» сюда не кладём. Это чужой граф и чужие веса. Доказательство языка — сеть, написанная на Kenga и обученная на Kenga.

```bat
bootstrap\bin\kenga-lite.exe run examples\ml\kenga_lm.kenga
bootstrap\bin\kenga-lite.exe run examples\ml\kenga_charlm.kenga
bootstrap\bin\kenga-lite.exe run examples\ml\kenga_char_talk.kenga
bootstrap\bin\kenga-lite.exe run examples\ml\kenga_trigram.kenga
scripts\bc-run.cmd examples\ml\kenga_trigram.kenga
scripts\kenga-birth.cmd
bootstrap\bin\kenga-lite.exe run examples\ml\kenga_mm_lm.kenga
bootstrap\bin\kenga-lite.exe run examples\ml\kenga_mm_gen.kenga
bootstrap\bin\kenga-lite.exe run examples\ml\kenga_mm_words.kenga
bootstrap\bin\kenga-lite.exe run examples\ml\kenga_mm_words_talk.kenga
scripts\bc-run.cmd examples\ml\kenga_birth.kenga
scripts\hf-pack.cmd
```

Word-LM на CPU lite: L=2, D=16, V=20. Дописывает `<s> kenga zhivet v yazyke .`  
Char-LM / mm-gen: тот же decoder, **CTX=12** (промпт `"kenga vidit "` больше не режется). Веса в `minds/kenga_char_*`. Talk грузит их без повторного обучения.  
Триграмма пишет обрывки синтаксиса. Ощутимый результат: `scripts\kenga-birth.cmd` — модель дописывает seed с `"fn add"`, кладёт `examples/ml/kenga_born.kenga`, lite выполняет и печатает **24** (`fact(add(2,3)-1)`). Не GGUF.

## Что это доказывает

Язык может описать и прогнать **архитектуру большого LM**, не Python и не PyTorch. Веса: `minds/kenga_lm_*.kt`, `minds/kenga_char_*.kt`.

Что ещё не доказано (и честно не будет за вечер):

| Нужно для «половины Grok» | Статус |
|---|---|
| Decoder / attn / FFN в `.kenga` | есть |
| Обучение next-token | есть, крошечный корпус |
| Словарь 50k–128k, контекст 4k–32k | нет |
| L≈32–64, D≈4096, MoE | нет, те же функции с другими D/L |
| Триллионы токенов | нет данных |
| GPU-ядра | нет (`docs/ROADMAP.md`) |
| Загрузка чужих весов (GGUF) | нет |

Без GPU и корпуса «половина меня» не появится, какой бы синтаксис ни был. Появится, когда этот же decoder жрёт реальные веса и железо.

## Лестница

1. XOR-MLP на list/f64 — `kenga_net.kenga` (алгоритм в языке, без tensor host).
2. 2-layer word-LM + CE — `word_lm.kenga`.
3. **Decoder GPT-формы** — `kenga_lm.kenga`.
4. **Char-LM на нашем исходнике** — `kenga_charlm.kenga` + `kenga_char_talk.kenga`.
5. **Триграмма на list/i64** — `kenga_trigram.kenga` ← вы здесь.
6. Больше D/L, байтовый/BPE словарь, длинный контекст.
7. GPU backend. Чужой GGUF — не доказательство.
8. Большая мультимодальная: git (пример) + Hugging Face `Kenga-ai/kenga-mm`. Сид и упаковка: `docs/HUGGINGFACE.md`.

Расти `fn D()` / `fn L()` / корпус в том же файле. Менять язык не нужно.
