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
  <img src="https://img.shields.io/badge/friends--ready-3.11-5b9dff?labelColor=12151a" alt="friends-ready"/>
</p>

---

## Для знакомых (30 секунд)

**Без Rust** — скачай бинарник с [Releases](https://github.com/GermannM3/kenga-lang/releases), клонируй репо (нужны `examples/`), положи `kenga` в `PATH`:

```powershell
git clone https://github.com/GermannM3/kenga-lang.git
cd kenga-lang
# скачай kenga-windows-x86_64.zip с Releases → kenga.exe в PATH
kenga demo
```

**Сборка из исходников** (если хочешь сам):

```bash
cargo install --path . --force --locked
kenga demo
```

**Rust-free lite** (C99, без хоста):

```bat
bootstrap\build.cmd
kenga run --lite examples\selfhost\struct_lite.kenga
```

Потом: `kenga chat minds/agent.km` → «смотри 5 1 6», «что будет завтра?»

Подробный питч: **[docs/FOR_FRIENDS.md](docs/FOR_FRIENDS.md)** · учить: **[docs/LEARN.md](docs/LEARN.md)** · справочник: **[docs/LANGUAGE.md](docs/LANGUAGE.md)** · упражнения: **[docs/EXERCISES.md](docs/EXERCISES.md)** · multimodal: **[docs/LIVING_MULTIMODAL.md](docs/LIVING_MULTIMODAL.md)** · chat/LM: **[docs/CHAT_AND_LM.md](docs/CHAT_AND_LM.md)** · Prophet без Rust: **[docs/PROPHET_LITE.md](docs/PROPHET_LITE.md)** · Tensor без Rust: **[docs/TENSOR_LITE.md](docs/TENSOR_LITE.md)** · план: **[docs/ROADMAP.md](docs/ROADMAP.md)**

Иконки `.kenga` в Cursor/VS Code — из Marketplace или локально:

```powershell
# Marketplace: ищи «Kenga Language» / publisher Kenga AI
.\editors\install-extension.cmd
```

---

## Зачем Kenga

Обычные стеки для ML — Python-клей вокруг C++/CUDA.  
**Kenga** — язык, где тензор, `ttl`, консолидация и агентный цикл часть семантики.

Сейчас — **3.11**: `kenga/compiler/more.kenga` ест for/struct/elif; emit пишет C без cargo (`scripts/freedom-smoke.cmd`).  
Карта: `docs/REPLACE_RUST.md`. Что в git: `docs/REPO.md`.  
Расширение: [VS Marketplace](https://marketplace.visualstudio.com/items?itemName=Kenga-ai.kenga) (`Kenga-ai.kenga`).  
Rust ещё держит GPU / production-scale path. **Python не нужен.** Linux/macOS/Git Bash: `docs/UNIX.md`.

<p align="center">
  <img src="assets/architecture.jpg" alt="Kenga architecture" width="900"/>
</p>

### Честно

| Да | Нет |
|---|---|
| Свой язык + VM + living memory | Не ChatGPT/LLM из коробки |
| Residual MLP + тензорный SGD | Не замена PyTorch |
| Lite без Rust (`bootstrap/` + **`kenga/`** compiler) | GPU; legacy `src/` пока в Releases |
| PPM/WAV → tensor bridges | Не pretrained CLIP/Whisper |
| emit-c tagged KVal + f64 | — |

---

## Установка

Linux / macOS / Git Bash → **`docs/UNIX.md`**.

### 1) Без Rust (рекомендуется друзьям)

1. Скачай архив с [Releases](https://github.com/GermannM3/kenga-lang/releases)  
   (`kenga-windows-x86_64.zip` / linux / macos).
2. Клонируй репо (нужны `examples/`, `minds/`):

```bash
git clone https://github.com/GermannM3/kenga-lang.git
cd kenga-lang
```

3. Положи `kenga` (или `kenga.exe`) в `PATH` или в корень клона.
4. Проверь:

```bash
kenga version
kenga demo
```

Rust-free lite (только C-компилятор):

```bat
bootstrap\build.cmd
kenga run --lite examples\hello.kenga
```

```bash
bash bootstrap/build.sh
kenga run --lite examples/hello.kenga
bash scripts/unix-smoke.sh
```

Пересобрать lite **из `.kenga` через emit-c** (chicken-egg):

```bat
bootstrap\rebuild-from-kenga.cmd
```

```bash
bash bootstrap/rebuild-from-kenga.sh
```

### 2) Сборка из исходников (нужен Rust 1.75+)

```bash
git clone https://github.com/GermannM3/kenga-lang.git
cd kenga-lang
cargo install --path . --force --locked
kenga version
```

## Быстрый старт

```bash
kenga demo
kenga run examples/ml/world_model.kenga
kenga run examples/ml/surprise_gate.kenga
kenga run examples/ml/tensor_core.kenga
kenga run examples/ml/mlp_tensor.kenga
kenga run examples/ml/train_sgd.kenga
kenga run examples/ml/autograd_tape.kenga
kenga run examples/ml/mlp_autograd.kenga
kenga run examples/ml/softmax_tape.kenga
kenga run examples/control_elif.kenga
kenga run examples/ml/vision_ppm.kenga
kenga run examples/ml/fusion.kenga
kenga run examples/neuromodel.kenga
kenga chat minds/agent.km
kenga run --lite examples/hello.kenga
kenga run --lite examples/native_lists.kenga
kenga run --lite examples/ml/living_multimodal.kenga
kenga chat --lite minds/multi.km
kenga run --lite examples/ml/tensor_core.kenga
kenga run --lite examples/ml/fusion.kenga
kenga run --lite examples/prophet.kenga
kenga run --lite examples/selfhost/arith.kenga
kenga run --lite examples/selfhost/loopfn.kenga
kenga run --lite examples/selfhost/for_lite.kenga
kenga run --lite examples/selfhost/struct_lite.kenga
kenga run --lite examples/selfhost/float_lite.kenga
kenga run --lite examples/selfhost/elif_lite.kenga
kenga run examples/selfhost/kenga_lite.kenga
kenga about
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
| Функции, if/else, while, for/in, break/continue | ✅ |
| Списки, ranges, struct, import | ✅ |
| Living `ttl` / `sweep`, events on/emit/pump | ✅ |
| Prophet memory + residual MLP + unroll | ✅ |
| `kenga chat` (русский диалог с mind) | ✅ |
| `save_mind` / `load_mind` | ✅ |
| `kenga demo` тур для друзей | ✅ |
| emit-c / build (i64/f64/list/str) | ✅ |
| Tape-autograd (`ag_*`) | ✅ |
| Self-host Kenga-lite → bytecode VM | ✅ |
| Полный self-host / LLVM / LLM | 🚧 |

---

## Команды CLI

```
kenga demo|tour
kenga about | which | version
kenga run <file.kenga>
kenga chat [mind.km] [--script f]
kenga eval | parse | compile | emit-c | build
```

---

## Структура

```
kenga-lang/
├── src/              # bootstrap compiler + VM + chat (Rust host)
├── bootstrap/        # Rust-free C99 kenga-lite
├── examples/         # demos + ml/ + selfhost/
├── editors/vscode/   # подсветка + иконка .kenga
├── minds/            # сохранённые world-model
├── stdlib/
├── docs/             # FOR_FRIENDS, LANGUAGE, SELFHOST
└── .github/          # CI + releases
```

Связанные проекты: [KengaAI_Engine](https://github.com/GermannM3/KengaAI_Engine), [The-Prophet](https://github.com/GermannM3/The-Prophet), [kengarust](https://github.com/GermannM3/kengarust).

---

## Лицензия

[MIT](LICENSE) © Kenga AI / [GermannM3](https://github.com/GermannM3)
