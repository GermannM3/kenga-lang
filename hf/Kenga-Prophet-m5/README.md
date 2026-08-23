---
license: apache-2.0
tags:
  - kenga
  - programming-language
  - small-model
  - transformer
  - verified-synthetic-corpus
  - compiler-in-the-loop
pipeline_tag: text-generation
---

# Kenga Prophet — M5 (same backbone as M4.2, data-scaled)

The experiment this model exists for: **keep the architecture and the
parameter budget fixed, change only the source and structure of the
training signal.**

M4.2 (identical ~838K-param backbone, trained on the small real Kenga
corpus) could not generalize: it produced **0 % compilable** programs on
unseen program templates. M5 is the same backbone trained on a
**compiler-verified synthetic corpus** built by the Kenga Corpus Factory.

## Headline results

| Metric | M4.2 (~838K, real corpus) | **M5 (~838K, factory corpus)** |
|---|---|---|
| Train tokens | 0.49 M | **1.81 M** |
| Held-out NT acc (template-split) | — | **89.26 %** |
| Generation compile rate (unseen templates) | 0 % | **100 %** |
| Generation run rate | 0 % | **100 %** |
| Value match (greedy) | 0 % | **20.0 %** |
| Value match pass@4 | 5 % | **32.5 %** |
| Zero-shot real-code NT transfer | — | **11.74 %** |

All generation numbers are measured on the **template-disjoint test
split**: programs whose structural template (source with integer
literals masked) never appears in training. Train/test template overlap:
**0**.

## What the model is

A pure-numpy transformer decoder trained from scratch on CPU:

- K=128 context, D=128, H=8 heads, L=6 layers → **~838K parameters**
- 128-token codec: full alphabet (`a-z A-Z 0-9 _`), BPE merges over real
  identifier words; numbers spelled digit-by-digit
- per-position causal LM objective
- Adam + global-norm gradient clipping (`clip=1.0`, `lr=0.002`) — the
  first M5 run diverged at step ~400 without clipping; clipping is part
  of reproducibility
- 2400 steps, batch 64, ~3 h on CPU

## The corpus (Kenga Corpus Factory)

14,585 generated programs in 4 families — arithmetic expression
functions, range loops, recursion (+ recursion↔iteration equivalents,
incl. Fibonacci), call chains:

- every program executed by the real `kenga-lite` runtime
  (compile → run → stdout); only passing programs kept
- **16,399 semantic-equivalence variants**, each re-verified to produce
  byte-identical stdout
- **10,343 mutation repair pairs** (broken source, failure class:
  run-fail / wrong-value / timeout) — released for a future Repair Model,
  *not* included in LM training
- split by template with literal masking: train 13,411 / test 1,174,
  overlap 0

## Honest notes

- **Previous "~1 % real-code accuracy" figures quoted for earlier models
  were invalid** due to an evaluation-script bug (logits mis-indexing)
  and must not be cited. The correct zero-shot real-code next-token
  transfer of this model is **11.74 %** (random ≈ 0.8 %).
- 89.26 % is in-distribution generalization to unseen templates of the
  same four program families. Transfer to human-written Kenga is real
  but modest (11.74 %) — that domain gap is the next research target.
- M4.2's "83 %" on real code is its own training set (memorization),
  not a held-out number.

## Usage

```python
import sys; sys.path.insert(0, 'code')
import kenchat
codec = kenchat.load_codec_vocab('kenga_full.pkl')
toks, src = kenchat.gen_tokens('fn add', 'mid_prophet_m5_w.txt',
                               max_tokens=200, codec=codec)
print(src)
```

`code/` contains the full pipeline: trainer (`train_m3.py`), corpus
factory (`corpus_factory.py`), template split (`corpus_split.py`),
generation eval (`corpus_eval.py`), real-code eval (`realcode_eval.py`),
inference helpers (`kenchat.py`). `data/` holds the exact train/test
JSONL splits used for this checkpoint.

## Citation-ish

Part of the Kenga Prophet ladder: Pico → M1 → M2 (linear) → M3/M3.x
(transformer + compiler verification) → M4 (scaling: size ≠ solution)
→ **M5 (data scaling: same params, dramatically better verified data)**.
