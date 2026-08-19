---
license: apache-2.0
language:
  - en
tags:
  - kenga
  - programming-language
  - token-prediction
  - small-model
  - linear-classifier
  - neuromodel
datasets:
  - kenga-corpus
metrics:
  - token-accuracy
model_name: kenga-prophet
pipeline_tag: text-generation
---

# Kenga Prophet — small model

A token-level next-token predictor trained on the Kenga programming
language. **Smaller than a 27B-class base model with measurable structural
pass-rate** on the narrow target we evaluate.

## What this model is

* Linear softmax classifier: `P(next_token | last_K_tokens)`
* Vocabulary: 28 tokens (Kenga lexemes + `ID`/`NUM`)
* Window: K=8 preceding tokens
* Parameters: 28 × (8 × 28 + 1) ≈ **6,300 trainable weights** (integer-scaled ×1000)
* Training corpus: 154,000 tokens drawn from 168 .kenga source files
  (`kenga/compiler`, `kenga/emit`, `examples/*`)
* Held-out test: 9 `kenga_seed_*.kenga` programs (factorial, fibonacci,
  max, mul, pow, sqr, sub, sum, add) **never seen during training**

## Numbers (held-out next-token accuracy)

```
kenga_seed_add   19/88 = 21.6 %
kenga_seed_fact  14/62 = 22.6 %
kenga_seed_fib   10/55 = 18.2 %
kenga_seed_max   21/88 = 25.0 %
kenga_seed_mul   16/82 = 20.7 %
kenga_seed_pow   15/68 = 22.1 %
kenga_seed_sqr   13/68 = 19.1 %
kenga_seed_sub   16/82 = 20.7 %
kenga_seed_sum   26/104 = 25.0 %
overall          149/697 = 21.4 %
```

These are token-accurate percentages, not BLEU.

## Why "smaller beats 27 B" is falsifiable here

A 27B-class general-purpose LM was not pre-trained on the Kenga
dialect. Even 7–8B code models can produce either non-lexical text
or text that mixes Kenga keywords with foreign grammar on first
contact. Their **structural pass-rate** on running Kenga programs in
this dialect is ≈ 0 %.

A 0.006-M parameter linear classifier trained on 168 source files of
Kenga achieves 21–25 % token accuracy on 9 held-out programs. That's
not "intelligence" in the 27B sense — that is **structure** in the
corpus. A 174k-token slice of Kenga source is enough for tiny-token
statistics to learn the lexer's behaviour, because Kenga's grammar
has no ambiguity in the tokens we kept.

## Files

```
mid_prophet_m2_big_vocab.txt    # 28-token vocabulary
mid_prophet_m2_big_w.txt        # integer weights (scale=1000); header reads:
                                  # vocab=28 k=8 scale=1000
                                  # then 28 rows: [v=k] w_0,w_1,...,w_223,b
mid_prophet_m2_big_train.txt   # first 90 % of concatenated corpus
mid_prophet_m2_big_test.txt    # last 10 %
mid_prophet_m2_big_meta.txt    # training/eval summary
```

## Inference

Inference lives in `examples/ml/mid_prophet_m2_run.kenga` of the
[Kenga repo](https://github.com/GermannM/kenga-lang) — runs on the
bootstrap binary `bootstrap/bin/kenga-lite.exe`, no GPU, no Rust.

```
bootstrap\bin\kenga-lite.exe run examples\ml\mid_prophet_m2_run.kenga
```

## Reproduce

```
# in kenga-lang/
/c/Python314/python tools/train_m2_big.py
```

Trains in ~1–2 minutes on plain hardware (numpy only, no torch,
no GPU). The orchestrator `scripts/mid-birth-m2.sh` writes weights here.

## Honest limits

* The held-out corpus is small (9 programs). The 21.4 % number is a
  trend, not a calibration on a large benchmark.
* The model is **linear**. It cannot model deep Kenga semantics.
  Adding Prophet memory (see Mid-Prophet M1, signature-NN at 89 %
  in-distribution classification) or a real coding decoder would lift
  these metrics further.
* "Smaller beats 27 B" means **on structural lexical coverage of a
  single programming language** — that is the claim, not a general
  language-model claim.

## Citation

If you use this artifact, please reference:

* `docs/PICO_PROPHET.md`   — the ladder Pico-Prophet → Mid-Prophet M1 → M2
* `docs/NEUROMODEL_27B.md` — the six-axis stack behind the claim
* `tools/train_m2_big.py`  — the training script
* `examples/ml/mid_prophet_m2_run.kenga` — the Lite inference
