# Roadmap до полной готовности

## 1. Свобода от Rust

| Статус | Шаг |
|---|---|
| ✅ | C99 bootstrap `kenga-lite` |
| ✅ | `kenga run --lite` + auto для `*_lite` / `selfhost/` |
| ✅ | Lite: f64, assert, else if, type/ttl ignore |
| ✅ | Lite + `more`: **`&&` `||` `!` `%`** (`logic_lite.kenga`) |
| ✅ | Lite + `more`: **`/* block comments */`** (`comments_lite.kenga`) |
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
| ✅ | Lite: **`foresee_n`** — `examples/unroll.kenga` без Rust |
| ✅ | Lite: **events** — `on "e"(x) { }` / `emit` / `pump` / `pending` / `listen` — `examples/agent.kenga` |
| ✅ | Lite: **`ag_*` / tape** — `examples/ml/autograd_tape.kenga`, `mlp_autograd.kenga` |
| ✅ | Lite: **`t_mse` / `t_patch_mean` / `t_linear_grad`** — `encoder_grad.kenga` |
| ✅ | Lite: **word-LM 2-layer + CE** (`t_log`/`ag_log`, `save_tensor`) — `word_lm.kenga` |
| ✅ | Self-host seed: **`write_file` / `emit_c_seed.kenga`** (Kenga → `.c` без Rust emit-c) |
| ✅ | Self-host step 9: **`kenga_more.kenga`** — f64 + lists + println/assert/round на bytecode VM |
| ✅ | C-lite: brace-match пропускает строки/комменты (чинит `kenga_lite.kenga` на lite) |
| ✅ | Каталог **`kenga/`** — каноническая замена `src/` (`compiler/more`, `emit/c_seed`) |
| ✅ | `docs/REPLACE_RUST.md` — карта модулей Rust → Kenga |
| ✅ | **`more.kenga`**: birth→24 + **`c_seed`/`expr_c`** (Kenga пишет `.c`, `\"`/`\\n` в парсере) |
| ✅ | **`more.kenga`**: tensor host builtins (`t_matmul` / `t_softmax` / `t_mean` / `t_mse`) |
| ✅ | **`more.kenga`**: Prophet host builtins (`remember` / `foresee` / `save_mind`) |
| ✅ | **`more.kenga`**: tape autograd (`ag_matmul` / `ag_backward` / `ag_step`) |
| ✅ | **`more.kenga`**: `t_set` / `save_tensor` / `load_tensor` + `mlp_autograd` on Kenga VM |
| ✅ | **`more.kenga`**: `load_ppm` / `load_wav` / `t_patch_mean` / `t_linear_grad` |
| ✅ | **`more.kenga`**: `print` / `sleep_ms` / `learn` / `predict` |
| ✅ | **`more.kenga`**: `unroll` / `remember_next` |
| ✅ | **`more.kenga`**: `foresee_n` |
| ✅ | **`more.kenga`**: `examples/ml/living_multimodal.kenga` (PPM+WAV → Prophet) |
| ✅ | **`more.kenga`**: `examples/ml/word_lm.kenga` (2-layer CE, ~3 мин) |
| ✅ | Forward `fn` call patches в `more.kenga` |
| ✅ | **Events** `on`/`emit`/`pump`/`pending` в `more.kenga` (`examples/agent.kenga`) |
| ✅ | **`kenga/emit/mini_codegen.kenga`** — parse tiny `.kenga` → C99 |
| ✅ | **`kenga/emit/core_c.kenga`** — while / if / for / list → C99 self-check |
| ✅ | **`kenga/emit/lower_c.kenga`** — agent / for / lists / **struct** / **elif** / **f64** / import → native C |
| ✅ | **`lower_c`**: `now_ms` |
| ✅ | **`kenga/emit/rt_kval.kenga` + `lower_kv.kenga`** — tagged KVal runtime + str/ord/hetero lists/events → `bootstrap/generated/` |
| ✅ | **`lower_kv`**: `now_ms` |
| ✅ | **`lower_kv`**: Memory как lite (`memory_config` / `remember` / `unroll` / `surprise` / `save_mind` / `load_mind`) |
| ✅ | **`lower_kv`**: Tensor как lite (`t_set`/`t_sub`/`t_mul`/`t_softmax`/`t_mean`/`load_ppm`/`save_tensor`/…) |
| ✅ | **`lower_kv`**: tape как lite (`ag_softmax`/`ag_log`/`ag_neg`/`ag_sum`/`ag_mul`/… + CE native) |
| ✅ | **`lower_kv`**: полный living (`living_multimodal` 24 эпохи, 3 сцены, save/load) — короткий `lower_kv_living` остаётся smoke |
| ✅ | **`lex_frag` / `parse_frag`** — куски lexer/parser компилятора → native C |
| ✅ | **`bc_src_c`** — parse → bytecode → C VM; **`kenga_net`**, **`kenga_birth`** (native пишет программу); **`scripts\bc-run.cmd`** |
| ✅ | **`bc_src_c`**: `print` / `sleep_ms` / `now_ms` |
| ✅ | **`bc_src_c`**: Tensor (`t_from` / `t_matmul` / `t_get` / `t_shape` / `tensor` / `t_fill`, ops 47–52) |
| ✅ | **`bc_src_c`**: Memory (`memory_config` / `learn` / `predict`, ops 53–55) |
| ✅ | **`bc_src_c`**: Prophet (`remember` / `unroll` / `save_mind` / `load_mind` / `foresee`, ops 63–71) |
| ✅ | **`bc_src_c`**: tape (`ag_clear` / `ag_param` / `ag_const` / `ag_matmul` / `ag_mse` / `ag_backward` / `ag_step`, ops 56–62) |
| ✅ | **`bc_src_c`**: полный tape (`ag_add`/`sub`/`mul`/`scale`/`relu`/`neg`/`transpose`/`reshape`/`exp`/`log`/`softmax`/`sum`/`value`/`grad`, ops 72–85) |
| ✅ | **`kenga/emit/{expr_c,control_c,core_c}.kenga`** + `scripts/freedom-smoke.cmd` |
| ✅ | Emit lite runtime: **`rt_types` / Prophet / tensor / tape / compiler / VM / selftest** — `kenga_lite.c` каркас |
| ✅ | CI job `lite` без cargo; release **требует** `kenga-lite` (cargo `kenga` ещё legacy в архиве) |
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
| ⬜ | GPU / production autograd (1660 свободна; lite всё ещё CPU f64) |

## 3. Multimodal

| Статус | Шаг |
|---|---|
| ✅ | `load_ppm` / `load_wav` / fusion |
| ✅ | Living multimodal: PPM+WAV → Prophet → `minds/multi.km` |
| ✅ | Chat intents + tiny word-LM seed (`tiny_lm.kenga`) |
| ✅ | 2-layer word-LM + CE + weight files (`word_lm.kenga`) |
| ⬜ | f32 / tiled matmul (efficiency под рост) |
| ✅ | **`kenga_net.kenga`** — 2-2-1 MLP + SGD на list/f64 (XOR), lite, bytecode→C и **`more` VM** |
| ✅ | **`kenga_lm.kenga`** — decoder (causal attn / FFN / RMS / residual), next-token на lite |
| ✅ | **`kenga_charlm.kenga`** — тот же decoder, корпус = наши `.kenga` (не GGUF) |
| ✅ | **`kenga_char_talk.kenga`** — generate из сохранённых весов |
| ✅ | **`kenga_trigram.kenga`** — char-триграмма на list/i64, корпус наш |
| ✅ | **`kenga_birth.kenga`** — suffix LM → `kenga_born.kenga` → run → 24 |
| ✅ | **`kenga_mm_lm.kenga`** / **`kenga_mm_talk.kenga`** — PPM+WAV → caption |
| ✅ | **`kenga_mm_gen.kenga`** — decoder пишет стебель цвета (kra/ze/si) |
| ✅ | **`kenga_mm_words`** — 3 подписи + `kenga zhivet v yazyke` (12 токенов); `more.kenga` гоняет XOR |
| ✅ | Карточка HF: `hf/kenga-seed/` + `scripts\hf-pack.cmd` |
| ⬜ | Larger generative LM (vocab/D/L + GPU); лестница в `docs/KENGA_LM.md` |
| ⬜ | Выкладка большой модели: git + **Hugging Face** `Kenga-ai/kenga-mm` |
| ◐ | GPU: OpenCL за `t_matmul` (Windows, оси ≥ 32). Не CUDA/wgpu |
| ⬜ | Pretrained encoders |

## 4. Docs / marketplace

| Статус | Шаг |
|---|---|
| ✅ | LEARN / LANGUAGE / UNIX / LIVING_MULTIMODAL / CHAT_AND_LM / INDEPENDENCE / REPLACE_RUST / HUGGINGFACE / … |
| ✅ | Marketplace: ручной Upload `.vsix` (без Azure PAT) |

Версия расширения: **3.13.0** (Marketplace). Язык в `kenga/` дальше; следующий VSIX — когда накопится в `editors/vscode`.
