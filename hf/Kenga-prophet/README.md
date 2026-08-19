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
  - baseline
  - v0-1
datasets:
  - kenga-corpus
metrics:
  - token-accuracy
model_name: kenga-prophet
pipeline_tag: text-generation
library_name: kenga
---

# Kenga Prophet — M2 baseline (v0.1)

The **first** Kenga-native trained model published externally. This
release is **immutable**: subsequent runs ship under separate model
names (`kenga-prophet-m2-k16`, …). Use this card as a permanent
point of reference for what "6,300 parameters + Kenga corpus" did
on the day of the first release.

## What this model is

* Linear softmax classifier: `P(next_token | last_K_tokens)`
* Vocabulary: **28 tokens** (Kenga lexemes + `ID`/`NUM`)
* Window: **K = 8** preceding tokens
* Parameters: **6,300 trainable weights** (28 × (8 × 28 + 1))
* Trained in **Python with numpy only**, no torch, no GPU
* Inference runs in **Kenga Lite more VM** (no GPU, no Rust)

If `6,300 / 27,000,000,000` sounds absurd, that's exactly the
proportion the user wants to track: small + structurally correct
versus big + general-purpose.

## Numbers (held-out next-token accuracy)

```
kenga_seed_add   19/88  = 21.6 %
kenga_seed_fact  14/62  = 22.6 %
kenga_seed_fib   10/55  = 18.2 %
kenga_seed_max   21/88  = 25.0 %
kenga_seed_mul   16/82  = 20.7 %
kenga_seed_pow   15/68  = 22.1 %
kenga_seed_sqr   13/68  = 19.1 %
kenga_seed_sub   16/82  = 20.7 %
kenga_seed_sum   26/104 = 25.0 %
overall          149/697 = 21.4 %
```

These are **token-accuracy numbers**, not BLEU. The "trick" is that
Kenga's grammar has no ambiguity in the 28-token codec, so even modest
per-token accuracy can produce **syntactically valid** continuations.

## Provenance (frozen at v0.1 release)

```
Kenga commit  :  993187398e8d5cda85e7c8a1fca44e648f87016a
Training   V  :  28
Context    K  :  8
Embedding features/V:  226 (K*V + bias)
Total params      :  6,300
Optimizer         :  Adam  (lr 5e-3, betas 0.9/0.999)
Epochs            :  60
Training corpus   :  168 .kenga source files, 154,000 tokens
Train/test split  :  first 90% / last 10%
Held-out program set : 9 kenga_seed_*.kenga programs

weights blob sha (16 hex):  28f7ef5c39008b52
vocab  blob sha            :  0246917ce1a8f263
train  blob sha            :  bc558fa4207b6db1
test   blob sha            :  d991ac600746b4c8
meta   blob sha            :  d13eb31ddcaba14b

Total on-disk size (all 5 artefacts):  ~ 580 KB
RAM at inference (Lite more VM):       ~ 1 MB
Wall-clock training time:             ~ 1–2 min  (numpy only)
Wall-clock per-token inference:       ~ 30 ms   (Lite more VM, single argmax)
Wall-clock full-prediction inference:  ~ 1 s    (Lite, 100 generated tokens)
CPU-only, no GPU required.
```

The `kenga-prophet` repo on Hugging Face is **immutable** at this
SHA: subsequent improvements go to `kenga-prophet-m2-k16`,
`kenga-prophet-m2-mlp`, etc. The v0.1 card stays as the **first**
point of reference.

**Weights format fix (v2 of this file):** the initial upload serialized
`\n` as literal backslash-n (single-line), which corrupted the weights,
vocab, and meta files for any consumer. This revision re-serializes them
with real newlines. All provenance values above (commit SHA, blob
hashes, params) are unchanged — this is a serialization fix, not a
retrain.

## Program-validity rate (honest, measured)

`tools/kenchat.py --probe` runs the model and feeds the generated
program to `kenga-lite`. Current result for v0.1:

```
compile-ok:    0/9 = 0.0%
run-ok:        0/9 = 0.0%
match value:   0/9 = 0.0%
```

The model cannot yet generate structurally valid programs: greedy
decoding always predicts `fn`, and 21% token accuracy means 79% of
tokens are wrong. This 0/9 is the honest baseline the ladder must
climb — see "What this model CANNOT do" below.

## What this model CAN do

* Given an 8-token prefix from Kenga source, predict the next
  token from the 28-token codec.
* Run in two or three minutes on a 1660-class GPU-less laptop
  (this is the entire training time).
* Be inspected losslessly: weights are integers in the file at
  `minds/mid_prophet_m2_big_w.txt`, vocabulary at
  `minds/mid_prophet_m2_big_vocab.txt`, training config in
  `minds/mid_prophet_m2_big_meta.txt`.

## What this model CANNOT do

* Open-ended chat on natural-language queries. It was trained on
  Kenga source, not on English.
* Pass-rate on long (multi-line) generation at this K=8 window is
  weak because **21% next-token accuracy means 79% wrong tokens**;
  one wrong token later in the program bleeds into syntactic
  breakage.
* Encode Kenga semantics. It is a next-token surface statistic.
  See Mid-Prophet M1 (`docs/PICO_PROPHET.md`) for a non-trained
  signature-based classifier that does better on identity
  classification tasks.

## Why this is genuinely Kenga-native and not "just another Python model"

| axis | this model | a Hugging Face PyTorch reference |
|---|---|---|
| Training | numpy only | PyTorch / JAX / TF |
| Optimizer | hand-rolled Adam (~80 lines) | torch.optim.Adam |
| Data pipeline | `walk('kenga/' + 'examples/')` + tokenize | datasets.load_dataset |
| Inference | `bootstrap/bin/kenga-lite.exe` reading weights from `minds/...` | HF pipeline / transformers |
| Runtime | Kenga Lite more VM (no Rust, no GPU) | CUDA / ROCm / CPU SIMD |
| Tokenisation | custom 28-token Kenga codec | BPE / WordPiece |
| File format | plain text integer weights | safetensors / ONNX |

Every stage is the same Kenga: tokenisation is built around the
Kenga grammar, inference runs on the `kenga-lite` binary that comes
with the language, and there is **no Python dependency in the
critical path** of inference. That is what makes this a
*Kenga-native* model and not "a Python model with Kenga data".

## Fixed sample predictions (token ids 0..27)

For random prefixes drawn from the held-out stream at
position 32 onward, the model picks the following tokens. These
are illustrative raw outputs, **not corrected**.

```
prefix  [13, 7, 14, 15, 7, ...]           predict token 7  (i64)
prefix  [11, 1, 26, 16, 12, ...]           predict token 10 (semicolon)
prefix  [0, 26, 9, 8, 7, 26, 14, ...]     predict token 11 ({)
```

These are toy outputs; the artefact here is **provenance and
ladder position**, not finished quality.

## Reproduce

```
# requires numpy only; on Windows:
git clone https://github.com/GermannM/kenga-lang
cd kenga-lang
python tools/train_m2_big.py
# produces minds/mid_prophet_m2_big_*.txt (~ 580 KB total)
```

```bash
# inference on a token stream:
minds/mid_prophet_m2_big_w.txt minds/mid_prophet_m2_big_vocab.txt  # explicit
bootstrap\bin\kenga-lite.exe run examples\ml\mid_prophet_m2_run.kenga
```

The orchestrator script `scripts/mid-birth-m2.sh` runs inference
against the 9 held-out programs and reports the aggregate accuracy.

## Citation

* `docs/PICO_PROPHET.md`     — the ladder Pico-Prophet → Mid-Prophet M1 → M2
* `docs/NEUROMODEL_27B.md`   — the six-axis stack behind the claim
* `tools/train_m2_big.py`    — the training script that produced this artefact
* `examples/ml/mid_prophet_m2_run.kenga` — the Lite inference harness
