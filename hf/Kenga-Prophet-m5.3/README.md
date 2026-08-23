---
license: apache-2.0
tags:
  - kenga
  - programming-language
  - small-model
  - transformer
  - semantic-binding
  - verified-synthetic-corpus
  - compiler-in-the-loop
pipeline_tag: text-generation
---

# Kenga Prophet — M5.3 (Semantic Binding / Factory v2)

**Same ~838K backbone as M4.2/M5/M5.2. Same training budget. Only the
training distribution changed: identifier and call-structure shortcuts
were removed from the verified corpus.**

Result: semantic binding (calling the right function among same-signature
distractors) went from broken to nearly solved — **binding compile ×17.6**
— with zero regression on the previous axes.

## The controlled A/B/C result

| Axis | Metric | M5.2 | **M5.3** |
|---|---|---|---|
| **A** factory/template generalization | compile / run | 92.5% | **100%** |
| | greedy match / pass@4 | 22.5 / 37.5% | 27.5 / 37.5% |
| **B** semantic binding (distractors) | compile | 5.6% | **98.6%** |
| | greedy match / pass@4 | 1.4 / 2.1% | 6.2 / 16.0% |
| **C** real-code generation | compile | 0/6 | 1/6 (17%) |
| | exact stdout match | 0/6 | 0/6 |

- Axis A test: unseen program templates, template-disjoint split.
- Axis B test (`bind`): a target function plus 1–2 **same-signature
  distractors**; `main` must call the target — the name is the only
  discriminator.
- Axis C: whole held-out human-written files; prompt = first fn block.

## What this proves

> The quality of a small model is determined not only by how much data
> it sees, but by **which causal-semantic dependencies the data
> distribution forces it to learn.**

M5.2 exposed the failure mode: trained on factory programs where `main`
always called `run()`, the model emitted structurally perfect programs
calling non-existent functions on real files (`run(10)` instead of
`fact(10)`). Factory v2 randomizes function/local names across a pool,
adds distractor functions, varies call patterns — and binding compiles
at 98.6% without any architectural change.

### The fact(14) story (why axis C match is 0)

M5.3's one successful real-file generation is more informative than its
failures: prompted with the first block of a factorial seed file, it
produced a **fully working factorial program**, correctly bound to
`fact`, compiled and ran — printing `14!` instead of the original's `5!`.
Exact-stdout matching on free continuation therefore conflates coding
ability with reproducing the file author's arbitrary constant choices.
`realgen` v2 (two-tier: body-completion with fixed args = fair match;
free continuation = valid-program rate only) is the next eval revision.

## Training

- numpy transformer decoder: K=128, D=128, H=8, L=6 → **~838K params**
- 128-token full-alphabet codec (`a-z A-Z 0-9 _`, BPE merges over real
  identifiers, digits spelled out)
- per-position causal LM, Adam + global-norm clip 1.0, LR 0.002,
  batch 64, 2400 steps (~3 h CPU)
- corpus v2: 14,550 verified programs in 5 families (arith / loop /
  rec incl. recursion↔iteration equivalents / chains / **bind**) +
  real Kenga files (90%/10% deterministic file-level split)
- every training program executed by the real `kenga-lite` runtime

## Repair signal (side experiment, rp0)

A repair model trained naively on 9,446 (broken → fixed) mutation pairs
reached **fixed@1 6.7% / pass@4 21.7%** on template-disjoint mutants —
*after* we fixed an evaluation bug where prompts containing a complete
`fn main` strangled generation at token 5 (stop-condition checked the
whole buffer). The naive setup lacks an explicit task boundary; a
marker-token variant (`<FIX>`) is the planned rp1 control.

## Honest notes

- Earlier "~1%" real-code figures quoted for M4-era models were invalid
  due to an evaluation-script bug; do not cite them.
- M4.2's "83%" real-code NT accuracy is train-set memorization, not
  held-out.
- M5.3 predates the run-manifest patch (no `_run.json`); later releases
  include git commit + corpus sha256 + config snapshots.

## Ladder position

```
M4.x   size ≠ solution
M5     verified synthetic data → template generalization (89.26%)
M5.2   +real code → transfer appears, binding shortcut exposed (5.6%)
M5.3   shortcut removed → binding solved (98.6%), A intact   ← this model
next   realgen-v2 tier-1, rp1 <FIX>, then Genesis v0 (gated)
```

Genesis entry gate status for this checkpoint: **binding gate PASSED**
(98.6% ≥ 40%); real-gen gate pending tier-1 recount (17% < 30%).

## Usage

```python
import sys; sys.path.insert(0, 'code')
import kenchat
codec = kenchat.load_codec_vocab('kenga_full.pkl')
toks, src = kenchat.gen_tokens('fn add', 'mid_prophet_m53_w.txt',
                               max_tokens=200, codec=codec)
print(src)
```

`data/` contains the exact template-disjoint splits (train/test JSONL)
and the 806-mutant repair eval set used for every number above.
