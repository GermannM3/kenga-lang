# Hugging Face

Большую мультимодальную модель кладём и в git (пример), и на HF. Скачанный GGUF туда не тащим.

## Сейчас

Сид уже multimodal: linear caption (`kenga_mm_lm`) и decoder с vis-bias (`kenga_mm_gen`). Карточка: `hf/kenga-seed/`. Собрать папку под заливку:

```bat
bootstrap\bin\kenga-lite.exe run examples\ml\kenga_mm_lm.kenga
scripts\hf-pack.cmd
```

Заливка (когда будет токен и решение лить сид или уже большую):

```bat
huggingface-cli upload Kenga-ai/kenga-seed-mm dist\hf-kenga-seed --repo-type model
```

Орг как у VS Code publisher: **Kenga-ai**.

## Когда модель большая

Тот же decoder (`kenga_dec.kenga`) и тот же vision→text (`kenga_mm_lm.kenga`), другие числа: D/L/V, корпус, GPU. Репозиторий: `Kenga-ai/kenga-mm`. В git — пример запуска, на HF — веса + карточка.

Не выкладываем чужие GGUF под именем Kenga.
