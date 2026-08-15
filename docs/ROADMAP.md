# Roadmap до полной готовности

## 1. Свобода от Rust

| Статус | Шаг |
|---|---|
| ✅ | C99 bootstrap `kenga-lite` |
| ✅ | `kenga run --lite` + auto для `*_lite` / `selfhost/` |
| ✅ | Lite: f64, assert, else if, type/ttl ignore |
| ✅ | Lite: **for / break / continue** (`native_lists.kenga` без Rust) |
| ✅ | Lite: forward `fn` calls, `str+str`, import, nested lists, `true`/`false` |
| ✅ | Self-host seed on lite: `arith` / `mini` / `iffy` / `loopfn` |
| ✅ | Unix: `bootstrap/build.sh`, `docs/UNIX.md`, `scripts/unix-smoke.sh` |
| ✅ | emit-c tagged KVal + chicken-egg |
| ✅ | README: install без cargo (Releases first) |
| ✅ | Lite: **Prophet Memory** + **save_mind / load_mind** (`.km` совместим с Rust) |
| ✅ | Lite: **`kenga-lite chat`** / `kenga chat --lite` — диалог без Rust |
| ✅ | Lite: **Tensor core** (`tensor` / `t_from` / matmul / ew / reshape / softmax…) — `examples/ml/tensor_core.kenga` |
| ✅ | Lite: **`load_ppm` / `load_wav` / `t_mean`** + `learn`/`predict`/`unroll`/`remember_next` — living multimodal без Rust |
| ✅ | Lite: **events** — `on "e"(x) { }` / `emit` / `pump` / `pending` / `listen` — `examples/agent.kenga` |
| ✅ | Lite: **`ag_*` / tape** — `examples/ml/autograd_tape.kenga`, `mlp_autograd.kenga` |
| ✅ | Lite: **`t_mse` / `t_patch_mean` / `t_linear_grad`** — `encoder_grad.kenga` |
| ✅ | Lite: **word-LM 2-layer + CE** (`t_log`/`ag_log`, `save_tensor`) — `word_lm.kenga` |
| ✅ | Self-host seed: **`write_file` / `emit_c_seed.kenga`** (Kenga → `.c` без Rust emit-c) |
| ✅ | Self-host step 9: **`kenga_more.kenga`** — f64 + lists + println/assert/round на bytecode VM |
| ✅ | C-lite: brace-match пропускает строки/комменты (чинит `kenga_lite.kenga` на lite) |
| ✅ | Каталог **`kenga/`** — каноническая замена `src/` (`compiler/more`, `emit/c_seed`) |
| ✅ | `docs/REPLACE_RUST.md` — карта модулей Rust → Kenga |
| ✅ | **`more.kenga` step 10**: for/break/continue, `xs[i]=`, else if, struct, import, `run_file` |
| ✅ | Forward `fn` call patches в `more.kenga` |
| ✅ | **Events** `on`/`emit`/`pump`/`pending` в `more.kenga` (`examples/agent.kenga`) |
| ✅ | **`kenga/emit/mini_codegen.kenga`** — parse tiny `.kenga` → C99 |
| ✅ | **`kenga/emit/core_c.kenga`** — while / if / for / list → C99 self-check |
| ✅ | **`kenga/emit/lower_c.kenga`** — agent / for / lists / **struct** / **elif** / **f64** / import → native C |
| ✅ | **`kenga/emit/rt_kval.kenga` + `lower_kv.kenga`** — tagged KVal runtime + str/ord/hetero lists/events → `bootstrap/generated/` |
| ✅ | **`lex_frag` / `parse_frag`** — куски lexer/parser компилятора → native C |
| ✅ | **`opcodes_c` + `bc_vm_c`** — opcodes + крошечный bytecode VM seed из Kenga |
| ✅ | **`kenga/emit/{expr_c,control_c,core_c}.kenga`** + `scripts/freedom-smoke.cmd` |
| ⬜ | Emit полный lite runtime → убрать ручной `kenga_lite.c` |
| ⬜ | VM на Kenga / native из своего codegen → убрать C host |
| ⬜ | Archive / удалить `src/` когда Releases = lite-only |

## 2. Ядро и тензоры

| Статус | Шаг |
|---|---|
| ✅ | matmul / reshape / transpose / exp / softmax / log |
| ✅ | Tape: relu/neg/transpose/reshape/exp/log/softmax |
| ✅ | Tape на lite (C99) |
| ✅ | Patch encoder + linear grad helpers на lite |
| ✅ | `save_tensor` / `load_tensor` / `write_file` / `read_file` |
| ⬜ | GPU / production autograd |

## 3. Multimodal

| Статус | Шаг |
|---|---|
| ✅ | `load_ppm` / `load_wav` / fusion |
| ✅ | Living multimodal: PPM+WAV → Prophet → `minds/multi.km` |
| ✅ | Chat intents + tiny word-LM seed (`tiny_lm.kenga`) |
| ✅ | 2-layer word-LM + CE + weight files (`word_lm.kenga`) |
| ⬜ | f32 / tiled matmul (efficiency под рост) |
| ⬜ | Larger generative LM (больше vocab/слоёв) |
| ⬜ | GPU backend (wgpu/CUDA) под 1660-class |
| ⬜ | Pretrained encoders |

## 4. Docs / marketplace

| Статус | Шаг |
|---|---|
| ✅ | LEARN / LANGUAGE / UNIX / LIVING_MULTIMODAL / CHAT_AND_LM / INDEPENDENCE / REPLACE_RUST / … |
| ✅ | Marketplace: ручной Upload `.vsix` (без Azure PAT) |

Версия: **3.11.0**.
