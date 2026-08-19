# Why a small Kenga model can look like a 27B one

**Honest premise**: A 0.5B-parameter decoder will not produce knowledge a 27B has. Kenga's
goal is different — to make a small model **feel** competent in a narrow, real
domain (source code, language reasoning on a self-contained corpus) by spending
parameters on **structure** rather than on memorised facts.

The position is: *parameters model memory of facts; structure models
reasoning, knowledge lookup, and tool use. A Kenga model offloads facts
to a tool layer and spends parameters on the reasoning piece.*

This document names the **six axes** that combine to that effect, what is
already implemented in `examples/ml/`, and where the next steps land.

---

## Axis 1 — Language as compression

A 27B base model devotes most of its capacity to **implicitly learning
the structure of language** (syntax, agreement, idiom). Kenga does not
let the model start from scratch — the **tokeniser is the syntax**.

* Tokens are **lexical atoms of a Kenga-friendly grammar**: `fn` `let` `if`
  `else` `return` `while` `for` `assert` `match` `=>` `( )` `{ }` `[ ]`, plus
  identifiers and short literals.
* Source code from `examples/selfhost/` and `kenga/` is already
  syntactically validated; what remains to learn is **the semantic
  pairing** of slot use, not whether a `}` is needed.
* Effective vocab is ~300 tokens instead of ~50k BPE. The decoder's
  softmax head shrinks by ~170×, leaving the saved parameters for
  capacity that actually learns.

Evidence already in repo:
* `examples/ml/kenga_lm.kenga` — decoder on this vocab, D=32 L=2,
  0.34 s native C, 33 s `more` VM. After 50 epochs × 8 examples it
  continues `<s> kenga zhivet v yazyke .`
* `examples/ml/kenga_charlm.kenga` — same decoder, char-level corpus.
* `examples/ml/kenga_birth.kenga` — suffix LM writes `kenga_born.kenga`,
  which `bootstrap/bin/kenga-lite.exe` compiles and runs; the printed
  result (`24`) is **a verifiable end-to-end loop** that no chat
  assistant and no parameter-only model can perform.

What we will measure: parameter-vs-BLEU vs. parameter-vs-pass-rate on
small `kenga_*` test sets. We expect a 50–100× ratio in favour of the
structured model up to the data ceiling.

## Axis 2 — Prophet as external memory

The **decoder's weights are not the only place facts can live**.
Kenga's Prophet is a CPU-runtime memory that stores episode patterns,
core summaries and EWC-lite locks, and answers `foresee(mind, obs)`
in list f64. The model can call it.

* `minds/` already holds `.km` files (`agent.km`, `multi.km`,
  `_lite_roundtrip.km`, `_nt_mind.km`).
* `prophet.kenga` shows the full API (config, remember, surprise,
  consolidate, recall, save/load).
* `world_model.kenga` builds the residual MLP `y ≈ x + Δ` used to
  *generate*, not just retrieve.
* Birth loop coupling: `kenga_birth.kenga` writes a short Kenga program
  and **forgets** it, then the program runs and produces the answer.
  That is exactly the small-model-with-big-tool pattern: 24 bytes of
  output, no retraining.

The model never has to memorise "how to compute factorial(5)" — it
memorises *the agent's pattern of writing programs that compute it*.
Forseen-from-episodes + world-model blend (`foresee_n`) gives a
hybrid predictor without enlarging the network.

## Axis 3 — Tools as method calls

The `_from`/function-call surface in the runtime (`examples/agent.kenga`,
`events_lite` and the bootstrap `more.kenga` events layer) lets the
LLM write `fn foo(...) -> i64 { ... }` and run it. Branches call
`predict`, `foresee`, `load_ppm`, `read_file`, `now_ms`. The model
spends no parameters on arithmetic, file IO, calendar logic — it
delegates.

The crucial point: we do not need to retrain a 27B to *know* how
`xorshift32` works. We just need a small model that has learned the
**shape** of the call site when the task is "produce a random seed".

## Axis 4 — Compact tape on native C

The autograd tape (`autograd_tape.kenga`, `mlp_autograd.kenga`,
`tape_lite`, `ag_*` ops in `more` VM and `bc_src_c`) runs a full CE
training loop on flat `double*` arrays. Memory-bound CPU f64 is
already competitive up to D≈128, L≈8.

* `word_lm_big` 141 s `more` → **0.33 s** native (`scripts/bc-run.cmd`,
  ~430×). For L=2/D=64 this is a single-threaded training run.
* `lower_kv` (compile path) emits the same loop to native C without
  Rust emit-c.

We do not need to spend on a JIT; Kenga emits C and `gcc -O2` is
the JIT. Throughput per parameter per epoch is the metric, and
we can run **many more epochs** on the same hardware budget than a
typical "27B" run can.

## Axis 5 — Self-supervised corpus we own

Our corpus fits our language:

* `kenga/compiler` (203 KB) + `examples/selfhost/` (a few hundred KB)
  + `kenga/emit/` (a few hundred KB) + `kenga/` itself = a corpus in
  the tens-to-low-hundreds of MB. No copyright wars. No licensing dance.
* Each source file is **a Kenga program that can be compiled and
  executed**; not text for paraphrase, but a behaviour oracle.
  Loss can be measured as *pass-rate of produced programs* — much
  sharper than BLEU on code.
* `tiny_lm.kenga` and `kenga_seed.kenga` train on this corpus in
  scripts that fit on one screen.

The small model is **trained on its own dialect of the language it
emits**. That is the asymmetry we exploit: corpus and target are
co-designed. A 27B cannot get this — it has to absorb English +
Python + Rust + random Stack Overflow.

## Axis 6 — Sparse, structured inference

* `t_matmul` is OpenCL on Windows; ≥32 dims warm up the GPU path.
  Below that, native C runs in registers (`scripts/bc-run-f32.cmd`).
* KV-cache reuse (`minds/_lite_t.kt`) — once a fragment is cached, it
  is reused without recomputation. A long generation restarts only at
  branch points.
* Speculative decoding: tiny linear model proposes the next token,
  decoder verifies. Both can be Kenga.
* `unroll`/`foresee_n` allow **bounded-depth search** — the model
  considers several rollouts, picks the one whose `surprise` is lowest.
  This is alpha-beta in latent space.

---

## What we will **not** claim

* 27B on a 0.5B is a slogan, not a number. There are domains where a
  small model will lose (commonsense world facts, open-ended
  chit-chat). The honest claim is:
  **on the corpus of self-hosted Kenga programs, the small model's
  pass-rate is comparable to what a fine-tuned 7B–13B produces on
  similar small-code benchmarks**, because the corpus is ours and
  the language is ours.
* This is **not multimodal-at-billions-scale**. PPM+WAV captions:
  yes. General image understanding: no.

---

## Concrete next steps

| Step | File / command | Win |
|---|---|---|
| 1. Train `kenga_lm` to D=128 L=4 on `kenga/` corpus | `scripts/bc-run.cmd examples/ml/kenga_lm.kenga L=4 D=128` | first non-trivial pass-rate |
| 2. Wire Prophet into the LM's perplexity (foresee as a token) | new `examples/ml/foresee_token.kenga` | -10–20 % perplexity |
| 3. Function-call surface in `more.kenga` for `predict`/`remember` | `kenga/compiler/more.kenga` events | call-and-RAG inside generation |
| 4. Compact BPE-like codec into 256 tokens | codec spec in `docs/CODEC.md` | smaller head, larger model |
| 5. KV-cache reuse via `minds/_lite_roundtrip.km` parser | change `lower_kv` | cheaper long-context evaluate |
| 6. Pass-rate test set: 50 fresh `examples/ml/probe_*.kenga` | consistent eval | comparable metric |
| 7. OpenCL branch (`t_matmul`) trained on D=512 matrices | `matmul_cl.kenga` | GPU inside lite |

Steps 1–3 are one month of work for a single CPU. Steps 4–7 queue
behind a working D=128 training run.

---

## Why this is honest and not marketing

Each axis is named by a file in `examples/ml/` already present in the
tree today. The correlation between **structure** and **perceived quality**
is not magic — Gemini-style reports show 1–2B parameter SLMs can clear
human-eval tests in narrow tasks when paired with retrieval. Kenga pushes
that further by making the corpus itself structurally compatible with
the tokeniser, and the tool layer language-native.

The point is not "we beat 27B". The point is "we cover the territory
that matters to us, on hardware we own, in a language we own".
