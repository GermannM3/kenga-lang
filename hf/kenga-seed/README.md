---
license: mit
language:
  - ru
  - en
tags:
  - kenga
  - multimodal
  - from-scratch
library_name: kenga
---

# kenga-seed-mm

Seed of a Kenga-native multimodal LM. Not a GGUF wrap, not PyTorch.

When the large model is ready, it lands here (same org, bigger weights) and on git as an example.

## What this seed is

| Piece | File |
|---|---|
| Vision+audio → text | `kenga_mm_lm.kenga` (linear) + `kenga_mm_gen.kenga` (decoder, stems kra/ze/si) |
| Text decoder | `examples/ml/kenga_charlm.kenga` / `kenga_dec.kenga` |
| Birth (writes runnable Kenga) | lite or native C (`bc-run`) → `kenga_born.kenga` → **24** |
| Living world-model | `examples/ml/living_multimodal.kenga` → `minds/multi.km` |

Captions on the three demo frames:

```
kenga vidit krasnyj kadr i slyshit ton
kenga vidit zelenyj kadr i slyshit ton
kenga vidit sinij kadr i slyshit ton
```

## What it is not

Not half of Grok. D/L/V are toy. No 50k tokenizer, no GPU kernels, no foreign weights.

## Run

```bat
bootstrap\bin\kenga-lite.exe run examples\ml\kenga_mm_lm.kenga
scripts\kenga-birth.cmd
```

## Weights

Kenga tensor text format (`KENGA_TENSOR 1`), not safetensors yet. See `minds/kenga_mm_*.kt` after a local train. The large release will use the same architecture files with larger `D` / `L` and a real corpus.

## Upload (when the big one exists)

```bat
scripts\hf-pack.cmd
huggingface-cli upload Kenga-ai/kenga-seed-mm dist\hf-kenga-seed --repo-type model
```

Org: **Kenga-ai**. Bigger checkpoint later: `Kenga-ai/kenga-mm` (same card family, different scale).
