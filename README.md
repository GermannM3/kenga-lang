<p align="center">
  <img src="assets/banner.jpg" alt="Kenga" width="640"/>
</p>

<p align="center">
  <strong>Kenga</strong> — язык программирования для живого ИИ<br/>
  память · предсказание · агенты · локальный инференс
</p>

<p align="center">
  <a href="https://github.com/GermannM3/kenga-lang/actions/workflows/ci.yml"><img src="https://github.com/GermannM3/kenga-lang/actions/workflows/ci.yml/badge.svg" alt="CI"/></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-2ee6d6?labelColor=12151a" alt="MIT"/></a>
  <a href="https://marketplace.visualstudio.com/items?itemName=Kenga-ai.kenga"><img src="https://img.shields.io/visual-studio-marketplace/v/Kenga-ai.kenga?label=VS%20Marketplace&color=5b9dff&labelColor=12151a" alt="VS Marketplace"/></a>
  <a href="https://github.com/GermannM3/kenga-lang/releases"><img src="https://img.shields.io/github/v/release/GermannM3/kenga-lang?include_prereleases&color=5b9dff&labelColor=12151a" alt="release"/></a>
  <p align="center">
  <img src="https://img.shields.io/badge/friends--ready-3.13-5b9dff?labelColor=12151a" alt="friends-ready"/>
</p>

---

## Для знакомых (30 секунд)

**Без Rust** — клонируй репо. Самый короткий путь:

```powershell
git clone https://github.com/GermannM3/kenga-lang.git
cd kenga-lang
bootstrap\build.cmd        # → bootstrap\bin\kenga-lite.exe (C99 host, ~140 KB)
bootstrap\bin\kenga-lite.exe chat
```

Диалог: «смотри 5 1 6», «что будет завтра?». Без скачиваний, без Rust, без Python.

Если хочешь полный CLI (kenga.exe из Releases) — он всё ещё содержит Rust-host для GPU/legacy path. Но **полный цикл разработки работает без Rust** через `kenga-lite` + `kenga-lite-gen` + любой C-компилятор (`gcc`, `cl`, `clang`).

**Native из `.kenga` без Rust** (новое) — `kenga-lite-gen` строит bytecode-рантайм в C, `gcc` собирает exe:

```bash
scripts/bc-run.cmd examples/ml/kenga_birth.kenga   # → bootstrap/generated/bc_one_out.exe
bc_one_out.exe                                      # → "24"
```

**Сборка из исходников** (если хочешь редактировать GPU-host или legacy `src/`):

```bash
cargo install --path . --force --locked
kenga demo
```

Книга (PDF и EPUB, как Z-система): **[book/](book/)** — `book/kenga_kniga_yantaras_v1.pdf`.

Подробный питч: **[docs/FOR_FRIENDS.md](docs/FOR_FRIENDS.md)** · учить: **[docs/LEARN.md](docs/LEARN.md)** · справочник: **[docs/LANGUAGE.md](docs/LANGUAGE.md)** · упражнения: **[docs/EXERCISES.md](docs/EXERCISES.md)** · multimodal: **[docs/LIVING_MULTIMODAL.md](docs/LIVING_MULTIMODAL.md)** · своя LM: **[docs/KENGA_LM.md](docs/KENGA_LM.md)** · Hugging Face: **[docs/HUGGINGFACE.md](docs/HUGGINGFACE.md)** · chat/LM: **[docs/CHAT_AND_LM.md](docs/CHAT_AND_LM.md)** · Prophet без Rust: **[docs/PROPHET_LITE.md](docs/PROPHET_LITE.md)** · Tensor без Rust: **[docs/TENSOR_LITE.md](docs/TENSOR_LITE.md)** · self-host: **[docs/SELFHOST.md](docs/SELFHOST.md)** · независимость: **[docs/INDEPENDENCE.md](docs/INDEPENDENCE.md)** · план: **[docs/ROADMAP.md](docs/ROADMAP.md)**

Иконки `.kenga` в Cursor/VS Code — из Marketplace или локально:

```powershell
# Marketplace: ищи «Kenga Language» / publisher Kenga AI
.\editors\install-extension.cmd
```

---

## Зачем Kenga

Обычные стеки для ML — Python-клей вокруг C++/CUDA.  
**Kenga** — язык, где тензор, `ttl`, консолидация и агентный цикл часть семантики.

Сейчас — **3.13**: `more` (Prophet + tape + decoder + ppm/wav) + `bc_src_c` (parse → bytecode → native C, opcodes 1–109) + своя LM (birth → **24**) + **Prophet** живая память + **decoder** (causal attn / FFN / RMS). Канон без Rust (`kenga/`).  
Карта: `docs/REPLACE_RUST.md`. Что в git: `docs/REPO.md`.  
**Python не нужен.** Linux/macOS/Git Bash: `docs/UNIX.md`.

**Self-host**: `bc_src_c.kenga` уже собирает свою VM в C, поддерживает opcodes 1–109. `kenga_lite.c` — каркас из 90 строк (CRT + `#include`). Один последний шаг — VM на самой Kenga без gcc/cl.

<p align="center">
  <img src="assets/architecture.jpg" alt="Kenga architecture" width="900"/>
</p>

### Нейромодель в репо

| Что | Где |
|---|---|
| **Prophet living memory** + `remember` / `predict` / `unroll` / `save_mind` | `examples/ml/world_model.kenga`, `more.kenga`, `docs/PROPHET_LITE.md` |
| **Tape-autograd** + CE + MLP (XOR, mlp_autograd) | `examples/ml/autograd_tape.kenga`, `mlp_autograd.kenga` |
| **Residual MLP** + `t_set`/`t_matmul`/`save_tensor` | `examples/ml/mlp_tensor.kenga` |
| **Decoder** (causal attn / FFN / RMS) — next-token | `examples/ml/kenga_lm.kenga`, `kenga_charlm.kenga`, `docs/KENGA_LM.md` |
| **Suffix-LM** пишет программу → run → **24** | `examples/ml/kenga_birth.kenga` (lite + native C) |
| **PPM+WAV → tensor → caption** + char-colored decode | `examples/ml/kenga_mm_lm.kenga`, `kenga_mm_words.kenga`, `docs/LIVING_MULTIMODAL.md` |
| Сид под Hugging Face | `hf/kenga-seed/`, `docs/HUGGINGFACE.md` |

### Честно

| Да | Нет |
|---|---|
| Свой язык + VM + living memory | Полный chicken-egg VM на Kenga без gcc/cl — остался один шаг |
| Residual MLP + тензорный SGD + decoder + Prophet | Большая LLM (vocab 50k+, L=32+, GPU) — не масштабировали |
| Lite без Rust (`bootstrap/` + `kenga/` compiler/emit) + **native из `.kenga` через bc_src_c** | GPU-ядро; legacy `src/` живёт в Releases |
| PPM/WAV → tensor bridges + decoder | Не pretrained CLIP/Whisper |
| Своя LM: birth → **24** (lite + native C), decoder видит кадр | Не Grok и не чужой GGUF |
| Сид под HF: `hf/kenga-seed/` | Большая модель ещё не залита |

---

## Установка

Linux / macOS / Git Bash → **`docs/UNIX.md`**.

### 1) Без Rust (рекомендуется друзьям)

Клонируй репо и собери lite-хост:

```bash
git clone https://github.com/GermannM3/kenga-lang.git
cd kenga-lang
bootstrap\build.cmd        # или bash bootstrap/build.sh
bootstrap\bin\kenga-lite.exe version
bootstrap\bin\kenga-lite.exe chat
```

Это **C99-компилятор + ~140 KB** → интерактивный диалог с Prophet-памятью. Без Rust, без Python.

Если хочешь полный CLI (kenga.exe с GPU/legacy-поддержкой) — скачай с [Releases](https://github.com/GermannM3/kenga-lang/releases). Он нужен только для GPU-пути; вся разработка идёт через `kenga-lite`.

**Native exe из своего `.kenga`** (без Rust, с gcc или cl):

```bat
bootstrap\build.cmd                         :: kenga-lite + kenga-lite-gen
echo examples\selfhost\argc_more.kenga > bootstrap\generated\_bc_path.txt
bootstrap\bin\kenga-lite-gen.exe run kenga\emit\bc_src_c.kenga
   → bootstrap\generated\bc_rt.inc.c, bc_one_out.c, ...
cl /O2 /TC bootstrap\generated\bc_one_out.c  :: MSVC
   → bootstrap\generated\bc_one_out.exe
bootstrap\generated\bc_one_out.exe           :: → свой native exe, opcodes 1–109
```

Пересобрать lite **из `.kenga` через emit-c** (chicken-egg):

```bat
bootstrap\rebuild-from-kenga.cmd
```

```bash
bash bootstrap/rebuild-from-kenga.sh
```

### 2) Сборка из исходников (нужен Rust 1.75+)

Только если хочешь работать над GPU-ядром или редактировать legacy `src/`. Для обычных задач — путь выше.

```bash
git clone https://github.com/GermannM3/kenga-lang.git
cd kenga-lang
cargo install --path . --force --locked
kenga version
```

## Быстрый старт

```bash
bootstrap\bin\kenga-lite.exe demo
bootstrap\bin\kenga-lite.exe chat
bootstrap\bin\kenga-lite.exe run examples/hello.kenga

# Нейромодель (всё в lite / native C, без Python):
bootstrap\bin\kenga-lite.exe run examples/ml/world_model.kenga     # Prophet + learning
bootstrap\bin\kenga-lite.exe run examples/ml/surprise_gate.kenga   # консолидация
bootstrap\bin\kenga-lite.exe run examples/ml/mlp_tensor.kenga      # residual MLP на тензоре
bootstrap\bin\kenga-lite.exe run examples/ml/mlp_autograd.kenga     # tape-autograd
bootstrap\bin\kenga-lite.exe run examples/ml/kenga_lm.kenga         # decoder / next-token
bootstrap\bin\kenga-lite.exe run examples/ml/kenga_mm_words.kenga   # PPM+WAV → текст
scripts\kenga-birth.cmd                                           # модель пишет программу → 24
scripts\bc-run.cmd examples/ml/kenga_birth.kenga                  # native exe
bootstrap\bin\kenga-lite.exe run examples/selfhost/kenga_lite.kenga

# Kenga пишет C сама (без Rust codegen):
bootstrap\bin\kenga-lite.exe run kenga\emit\bc_src_c.kenga
scripts\freedom-smoke.cmd
```
Минимальная программа:

```kenga
fn main() -> i64 {
    println("hello from kenga");
    return 0;
}
```

World-model:

```kenga
fn main() {
    let mind = memory();
    learn(mind, [1, 0, 0], [0, 1, 0]);
    println(round(predict(mind, [1, 0, 0])));
    println(unroll(mind, [1, 0, 0], 3));
}
```

---

## Что уже есть

| Возможность | Статус |
|---|---|
| **Установка без Rust, без Python** — `bootstrap\build.cmd` + `kenga-lite` | ✅ |
| Функции, if/else, while, for/in, break/continue | ✅ |
| Списки, ranges, struct, import | ✅ |
| Living `ttl` / `sweep`, events on/emit/pump | ✅ |
| **Prophet** — living memory + unroll + `save_mind`/`.km` | ✅ |
| **Tape-autograd** (`ag_*`) + MLP на списках | ✅ |
| **Тензорный SGD** + `t_set`/`t_matmul`/`save_tensor`/`load_tensor` | ✅ |
| **Decoder** — causal attn / FFN / RMS / residual + CE | ✅ |
| **Suffix-LM**: пишет программу → run → **24** (lite + native C) | ✅ |
| **PPM+WAV → tensor → caption** + char-colored decode (k/z/s) | ✅ |
| **kenga chat** — русский диалог с mind (Prophet-driven) | ✅ |
| **kenga demo** тур | ✅ |
| **`lower_c`**: Kenga → native C (agent/for/lists/struct/elif/float) | ✅ |
| **`lower_kv` + `rt_kval`**: tagged KVal (str/ord/lex_frag/agent) | ✅ |
| **`bc_src_c`**: parse → bytecode → C-рантайм → `bc_one_out.exe` через `gcc`/`cl` (opcodes 1–109: print/sleep/now_ms, tensor, Prophet, tape, `argc`/`arg`/`file_exists`/`read_line`) | ✅ |
| Native из `.kenga` без Rust: `scripts/bc-run.cmd` | ✅ |
| Self-host lite-host: `bootstrap\rebuild-from-kenga.cmd` (Kenga → C → kenga-lite) | ✅ |
| Полный self-host: VM на Kenga, без gcc/cl | 🚧 — последний шаг лестницы (`docs/SELFHOST.md`) |
| Большая LLM (vocab 50k+, L=32+, GPU-ядро) | 🚧 — масштаб, не лестница |



## Команды

`bootstrap\bin\kenga-lite.exe` (без Rust):

```
kenga-lite run <file.kenga>
kenga-lite chat [mind.km]
kenga-lite eval '<src>'
```

`scripts\bc-run.cmd <file.kenga>` (native exe через gcc/cl).

`kenga` (полный CLI из Releases, GPU/legacy):

```
kenga demo | tour | about | version
kenga run <file.kenga>
kenga chat [mind.km] [--script f]
kenga eval | parse | compile | emit-c | build
```

---

## Структура

```
kenga-lang/
├── kenga/            # канон: compiler/ + emit/ (замена src/)
├── bootstrap/        # Rust-free C99 kenga-lite host
├── src/              # legacy Rust (Releases), новый код сюда не кладём
├── examples/         # demos + ml/ + selfhost/
├── hf/kenga-seed/    # карточка сида под Hugging Face
├── editors/vscode/   # подсветка + иконка .kenga
├── minds/            # сохранённые world-model и веса .kt
├── stdlib/
├── docs/             # LEARN, KENGA_LM, HUGGINGFACE, …
└── .github/          # CI + releases
```

Связанные проекты: [KengaAI_Engine](https://github.com/GermannM3/KengaAI_Engine), [The-Prophet](https://github.com/GermannM3/The-Prophet), [kengarust](https://github.com/GermannM3/kengarust).

---

## Лицензия

[MIT](LICENSE) © Kenga AI / [GermannM3](https://github.com/GermannM3)
