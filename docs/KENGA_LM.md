# Своя языковая модель на Kenga

Половина Grok — это не другой алгоритм. Это **тот же decoder** (внимание, residual, FFN, норма, LM-head) с другими числами: ширина, глубина, словарь, данные, GPU.

Kenga уже выражает эту машину. Два файла:

- `examples/ml/kenga_lm.kenga` — decoder на закрытом словесном словаре
- `examples/ml/kenga_charlm.kenga` — тот же decoder, корпус = наш `.kenga` (`kenga_seed.kenga` через `read_file`)

Скачанный GGUF из папки «kenga ai» сюда не кладём. Это чужой граф и чужие веса. Доказательство языка — сеть, написанная на Kenga и обученная на Kenga.

```bat
bootstrap\bin\kenga-lite.exe run examples\ml\kenga_lm.kenga
bootstrap\bin\kenga-lite.exe run examples\ml\kenga_charlm.kenga
```

Word-LM на CPU lite: L=2, D=16, V=20, ~6k весов. Дописывает `<s> kenga zhivet v yazyke .`  
Char-LM читает `examples/ml/kenga_seed.kenga`, строит charset, учит next-char, пишет с `"fn add"` что-то вроде `fn add(y);` плюс скобки языка. Не Grok и не скачанный GGUF — сеть из этого файла, корпус из нашего `.kenga`.

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
4. **Char-LM на нашем исходнике** — `kenga_charlm.kenga` ← вы здесь.
5. Больше D/L, байтовый/BPE словарь, длинный контекст.
6. GPU backend. Чужой GGUF — не доказательство.

Расти `fn D()` / `fn L()` / корпус в том же файле. Менять язык не нужно.
