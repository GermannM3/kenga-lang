# Examples for ML friends

| File | Story |
|---|---|
| `world_model.kenga` | residual MLP учит физику агента, accuracy + unroll |
| `surprise_gate.kenga` | surprise → episodic remember → sleep |
| `../neuromodel.kenga` | полный пайплайн train/sleep/predict/events |
| `../deep_train.kenga` | жёсткая тренировка → `minds/agent.km` |
| `../selfhost/kenga_lite.kenga` | компилятор подмножества на чистой Kenga |

```bash
kenga run examples/ml/world_model.kenga
kenga run examples/ml/surprise_gate.kenga
kenga demo
```
