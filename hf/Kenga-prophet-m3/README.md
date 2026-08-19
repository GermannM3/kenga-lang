---
license: apache-2.0
tags:
  - kenga
  - programming-language
  - small-model
  - transformer
  - token-prediction
  - m3
pipeline_tag: text-generation
---

# Kenga Prophet — M3 (transformer)

First non-linear model in the ladder. Single-axis improvement over
M2.1: **a real transformer decoder** (learned embeddings, causal
multi-head attention with QKV projections, tanh FFN, residuals,
proper backprop) instead of the linear softmax.

## What changed from M2.1

| axis | M2.1 (K=16) | M3 |
|---|---|---|
| Architecture | linear softmax | **transformer decoder** |
| K (context window) | 16 | **32** |
| Heads | — | 4 (head_dim 8) |
| Total parameters | 12,540 | **~11,100** |
| Held-out token accuracy | 23.4 % | **71.9 %** |
| Training time | ~ 1–2 min | ~ 5 min (numpy) |
| Disk size | ~ 580 KB | ~ 150 KB |

## Numbers (held-out next-token accuracy)

```
kenga_seed_add   49/64  = 76.6 %
kenga_seed_fact  25/34  = 73.5 %
kenga_seed_fib   20/27  = 74.1 %
kenga_seed_max   36/60  = 60.0 %
kenga_seed_mul   41/54  = 75.9 %
kenga_seed_pow   29/40  = 72.5 %
kenga_seed_sqr   27/40  = 67.5 %
kenga_seed_sub   42/54  = 77.8 %
kenga_seed_sum   54/76  = 71.1 %

overall:         323/449 = 71.9 %
```

This is a **3.3x** jump over the linear models (21–23 %), achieved
with ~11k parameters trained in ~5 minutes on CPU.

## Weights format (different from M2.x)

M3 uses named tensors instead of per-class rows:

```
vocab=28 k=32 d=32 h=4 head=8 scale=1000 arch=transformer
[E_tok] shape=[28, 32]  78,-222,-584,...
[E_pos] shape=[32, 32]  ...
[Wq] shape=[32, 32]     ...
[Wk] [Wv] [Wo] [W1] [b1] [W2] [b2] [Wout] [bout]
```

All values are ints scaled ×1000 (divide by 1000 at load).

## Provenance

```
Kenga commit  :  ddd48da
Training   V  :  28
Context    K  :  32
Embed dim  D  :  32
Heads      H  :  4  (head_dim 8)
Total params   :  ~11,100
Optimizer      :  Adam  (lr 5e-3, betas 0.9/0.999)
Steps          :  2400 mini-batches (batch 256, K=32 window)
Training corpus:  Kenga source (held-out = 9 kenga_seed_*.kenga)
```

## Honest limits (program-validity rate)

`tools/kenchat.py --probe --model m3` measures whether the model can
generate a program that **compiles, runs, and prints the expected
value** through `kenga-lite`:

```
compile-ok:    0/9 = 0.0%
run-ok:        0/9 = 0.0%
match value:   0/9 = 0.0%
```

Token accuracy is high (72 %) but autoregressive generation still
drifts on the long tail (weak `)`, `return`, `=`, `NUM` tokens ~20–60 %),
so no generated program is yet structurally valid. This is the honest
baseline for the next ladder rung (bigger M3, or grammar-constrained
decoding).

## Reproduce

```
/c/Python314/python tools/train_m3.py      # retrain (~5 min)
/c/Python314/python tools/kenchat.py --probe --model m3
```