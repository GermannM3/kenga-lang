# Exercises

Проверяй через `kenga run --lite` (или полный `kenga run`). Ответы не подглядывай в `examples/` сразу — сначала сам.

## E1. Арифметика

Напиши `examples/exercises/e01_sum.kenga`: сумма чисел от 1 до 100, `println`, `return 0`.

Ожидай: `5050`.

```bat
kenga run --lite examples\exercises\e01_sum.kenga
```

## E2. Список

Список `[3, 1, 4, 1, 5]`, найди максимум циклом, напечатай.

Шпаргалка по диалекту: `examples/selfhost/lists_lite.kenga`.

## E3. Struct

`struct Vec2 { x, y }`, функция `len2(v)` = x²+y², проверь на `(3,4)` → `25`.

Lite: `examples/selfhost/struct_lite.kenga`.  
Бонус: прогони через lowerer — `kenga/emit/lower_c.kenga` уже умеет struct → C.

## E4. События

Мини-агент: `on "tick"(n) { … }`, `emit` / `pump`. Ожидай цепочку как в `examples/agent.kenga`.

## E5. Тензор

`t_from([2,2], [1,0,0,1])` — unit. Умножь на `t_from([2,1],[3,4])`, проверь результат `[3,4]`.

## E6. SGD

С нулевого `W` выучи отображение `[1,0] → [2]` одним рядом `t_sgd_step` в цикле (см. `train_sgd.kenga` как шпаргалку после попытки).

## E7. Картинка

`load_ppm("examples/ml/assets/dot.ppm")` → `t_mean` → три числа около `0.5`.  
Тот же путь на VM `more`: `examples/selfhost/vision_more.kenga`. Living цикл целиком: `examples/ml/living_multimodal.kenga` на той же VM.

## E8. Fusion

Сложи image embedding + `t_from([3],[0.1,0.1,0.1])`, получи scalar через `t_matmul` с `[1,1,1]`.

---

## E9. Birth

Запусти `scripts\kenga-birth.cmd`. Ожидай: модель пишет `examples/ml/kenga_born.kenga`, запуск печатает **24**.

Прочитай `kenga_birth.kenga` — это suffix-LM, не GGUF.

## E10. Видишь и слышишь

`examples/ml/kenga_mm_lm.kenga` — три подписи про цвет кадра и тон. Прогони, сверь три строки.

## E11. Decoder видит кадр

`examples/ml/kenga_mm_gen.kenga` — стебель **kra / ze / si**.  
`examples/ml/kenga_mm_words.kenga` — цвет одним словом + `kenga zhivet v yazyke`.

Бонус: `scripts\bc-run.cmd examples\ml\kenga_birth.kenga` — birth без lite VM, потом `kenga-lite run examples\ml\kenga_born.kenga` → 24.

Дальше: `docs/KENGA_LM.md` · `docs/HUGGINGFACE.md` · `docs/ROADMAP.md`.
