# Examples for ML friends

| File | Story |
|---|---|
| `living_multimodal.kenga` | **PPM+WAV → Prophet → sleep → `minds/multi.km`** |
| `kenga_charlm.kenga` | decoder на наших `.kenga` (seed + selfhost) |
| `kenga_char_talk.kenga` | generate из `minds/kenga_char_*` |
| `kenga_trigram.kenga` | char-триграмма list/i64, без Tensor |
| `kenga_birth.kenga` | suffix LM пишет `kenga_born.kenga` (запуск → 24) |
| `kenga_mm_lm.kenga` | PPM → caption, сид для Hugging Face |
| `kenga_dec.kenga` | общие блоки decoder (import) |
| `kenga_lm.kenga` | decoder GPT-формы, закрытый словарь |
| `tiny_lm.kenga` | tiny word-LM на tape (генерация фразы) |
| `world_model.kenga` | residual MLP учит физику агента |
| `surprise_gate.kenga` | surprise → episodic remember |
| `tensor_core.kenga` | matmul / add |
| `mlp_tensor.kenga` | dense `W@x+b` |
| `train_sgd.kenga` | явный `t_sgd_step` |
| `vision_ppm.kenga` | `load_ppm` → mean RGB |
| `fusion.kenga` | image + wav + text vector |
| `../neuromodel.kenga` | полный train/sleep/predict |
| `../deep_train.kenga` | → `minds/agent.km` |
| `../selfhost/*_lite.kenga` | Rust-free C99 bootstrap |

```bash
kenga run examples/ml/living_multimodal.kenga
kenga chat minds/multi.km
kenga run examples/ml/kenga_charlm.kenga
kenga run examples/ml/kenga_trigram.kenga
kenga run examples/ml/tiny_lm.kenga
kenga run examples/ml/fusion.kenga
kenga run examples/ml/train_sgd.kenga
kenga run --lite examples/selfhost/struct_lite.kenga
kenga demo
```

Подробнее: [docs/LIVING_MULTIMODAL.md](../../docs/LIVING_MULTIMODAL.md) · [docs/CHAT_AND_LM.md](../../docs/CHAT_AND_LM.md).
