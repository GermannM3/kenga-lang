# Living multimodal

Kenga умеет **видеть + слышать + жить**: PPM/WAV → вектор наблюдения → Prophet world-model → surprise → sleep → `.km`.

Это не CLIP и не Whisper. Это маленький живой цикл на **CPU**, который уже бежит на слабом ПК (в т.ч. с GTX 1660 в корпусе — GPU пока не нужен).

## Быстрый прогон

```bash
# полный VM (Rust) или lite (C99, без Rust):
kenga run examples/ml/living_multimodal.kenga
kenga run --lite examples/ml/living_multimodal.kenga
kenga chat --lite minds/multi.km
# тот же файл на VM more:
bootstrap\bin\kenga-lite.exe run kenga\compiler\more.kenga
```

Ожидаемо:
- loss падает за ~24 эпохи
- odd-кадр (dot + beep) проходит surprise gate
- `consolidate` складывает эпизоды в core
- появляется [`minds/multi.km`](../minds/multi.km)

## Что внутри obs (dim=9)

| Срез | Откуда |
|---|---|
| 0..2 | mean RGB кадра (`load_ppm` → `t_mean`) |
| 3..5 | audio mean / energy / peak (`load_wav`) |
| 6..8 | time cues (индекс сцены) |

Кадры: `examples/ml/assets/frame{0,1,2}.ppm` + `tone{0,1,2}.wav`.  
Fusion-smoke: `beep.wav` + `dot.ppm` (`examples/ml/fusion.kenga`).

## Живость

1. **Учится** переходам сцена→сцена (`learn` / `remember_next`)
2. **Удивляется** чужому кадру (`surprise` / `foresee`)
3. **Спит** (`consolidate`, EWC-lite locks)
4. **Помнит** на диске (`save_mind` / `load_mind`)
5. **Говорит** через `kenga chat` (dim=9)

## Про 1660 и «большую» модель

Сейчас всё на **CPU f64**. CUDA/wgpu в языке ещё нет — карта простаивает, и это нормально при сотнях параметров.

Дальше по росту:
1. f32 + tiled matmul (всё ещё CPU, но быстрее/меньше RAM)
2. шире hidden / dim в Prophet
3. GPU backend за тем же `t_matmul`, когда модель перестанет комфортно жить на naive f64

Честный критерий «слабому железу ок»: demo заканчивается за секунды, mind < 100KB, chat стартует без OOM.

## Текст из кадра и звука

`examples/ml/kenga_mm_lm.kenga` — linear head, три готовые подписи.  
`examples/ml/kenga_mm_gen.kenga` — по буквам, стебель.  
`examples/ml/kenga_mm_words.kenga` — цвет одним токеном, три подписи, плюс `kenga zhivet v yazyke`. Карточка: `docs/HUGGINGFACE.md`.
