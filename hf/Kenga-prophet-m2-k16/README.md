---
license: apache-2.0
tags:
  - kenga
  - programming-language
  - small-model
  - linear-classifier
  - token-prediction
  - m2-1
pipeline_tag: text-generation
---

# Kenga Prophet — M2.1 (K=16)

Single-axis improvement over the v0.1 baseline: **K = 16** instead
of 8. Everything else is identical (vocab=28, linear softmax,
6,300 → 12,540 parameters).

## What changed from v0.1

| axis | v0.1 | M2.1 |
|---|---|---|
| K (context window) | 8 | **16** |
| Total parameters | 6,300 | **12,540** |
| Held-out token accuracy | 21.4 % | **23.4 %** |
| In-distribution accuracy | 41.1 % | 41.1 % |
| Training time | ~ 1–2 min | ~ 1–2 min |
| Disk size | ~ 580 KB | ~ 580 KB |

## Numbers (held-out next-token accuracy)

```
kenga_seed_add   18/80  = 22.5 %
kenga_seed_fact  13/50  = 26.0 %
kenga_seed_fib    9/43  = 20.9 %
kenga_seed_max   20/76  = 26.3 %
kenga_seed_mul   15/70  = 21.4 %
kenga_seed_pow   13/56  = 23.2 %
kenga_seed_sqr   11/56  = 19.6 %
kenga_seed_sub   15/70  = 21.4 %
kenga_seed_sum   25/92  = 27.2 %
overall          139/593 = 23.4 %
```

## Provenance (frozen at M2.1 release)

Single axis changed: K = 8 → 16. Same: vocab=28, arch=linear softmax,
optimizer=Adam (lr 5e-3, betas 0.9/0.999), epochs=60, training data
hash, kenga commit, kenga-lite runtime, etc.

**Weights format fix (v2 of this file):** the initial upload serialized
`\n` as literal backslash-n (single-line). That corrupted the file for
any consumer. The weights in this revision are re-serialized with real
newlines. Provenance (commit SHAs) is unchanged.

## Honest limits

The **program-validity rate** for M2.1 is currently **0/9** (greedy
generation always predicts `fn`, so generated programs never compile).
Token accuracy (23.4 %) is not yet high enough to generate structurally
valid code. This is the known gap the ladder is meant to close:
M2.2 (hidden layer) and M2.3 (BPE codec) target exactly this.
Run the probe:

```
python tools/kenchat.py --probe --model k16
```

## Reproduce / ladder

* **M2.0 (v0.1)**: `GermannM/kenga-prophet` — K=8, 6,300 params, 21.4 %
* **M2.1 (this repo)**: `GermannM/kenga-prophet-m2-k16` — K=16, 12,540 params, 23.4 %

Future M2.x artefacts will keep the same axis-decoupled layout:
each varies only one hyperparameter at a time. Series budget:

* M2.2 — structure: introduce a tiny hidden layer (FFN) ~ 50K params
* M2.3 — code: train on **BPE-style 64-token** codec instead of hand-tuned 28
* M3.x — accuracy ladder once accuracy clears 30 %

