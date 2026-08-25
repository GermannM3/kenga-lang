<p align="center">
  <img src="assets/banner.jpg" alt="Kenga" width="640"/>
</p>

<p align="center">
  <strong>Kenga</strong> — язык со своим компилятором, VM и выходом в C99<br/>
  MIT · ~140 KB host · без pip и cargo
</p>

<p align="center">
  <a href="https://github.com/GermannM3/kenga-lang/actions/workflows/ci.yml"><img src="https://github.com/GermannM3/kenga-lang/actions/workflows/ci.yml/badge.svg" alt="CI"/></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-2ee6d6?labelColor=12151a" alt="MIT"/></a>
  <a href="https://marketplace.visualstudio.com/items?itemName=Kenga-ai.kenga"><img src="https://img.shields.io/visual-studio-marketplace/v/Kenga-ai.kenga?label=VS%20Marketplace&color=5b9dff&labelColor=12151a" alt="VS Marketplace"/></a>
  <a href="https://github.com/GermannM3/kenga-lang/releases"><img src="https://img.shields.io/github/v/release/GermannM3/kenga-lang?include_prereleases&color=5b9dff&labelColor=12151a" alt="release"/></a>
</p>

---

## 30 секунд

```powershell
git clone https://github.com/GermannM3/kenga-lang.git
cd kenga-lang
bootstrap\build.cmd
bootstrap\bin\kenga-lite.exe run examples\hello.kenga
```

Печатает `hello from kenga` и `42`. Хост — C99, ~140 KB. Rust и Python для этого не нужны.

```kenga
fn main() -> i64 {
    println("hello from kenga");
    return 0;
}
```

Справочник: [docs/LANGUAGE.md](docs/LANGUAGE.md) · спека: [docs/SPEC.md](docs/SPEC.md) · self-host: [docs/SELFHOST.md](docs/SELFHOST.md). Подсветка `.kenga` — [Marketplace](https://marketplace.visualstudio.com/items?itemName=Kenga-ai.kenga) (3.13.0). Unix: [docs/UNIX.md](docs/UNIX.md).

GitHub Linguist ещё не знает `.kenga` (порог: тысячи публичных файлов у разных авторов). TextMate-грамматика уже отдельно: [kenga-grammar](https://github.com/GermannM3/kenga-grammar). Черновик PR: `editors/linguist/`. Полоска «Kenga» на github.com появится после их релиза, не после нашего пуша.

---

## Что это

Язык. Компилятор `kenga/compiler/more.kenga` написан на Kenga. Байткод-VM. Emit в C (`lower_c`, `bc_src_c`). Новый код — в `kenga/` и `bootstrap/`, не в `src/` (там legacy Rust).

В языке же: тензоры, tape-autograd, Prophet-память (`remember` / `foresee`), события. Это библиотека рантайма, не отдельный продукт.

**Вместо Python для маленького LM.** Decoder, XOR-MLP и birth (модель пишет `.kenga` → тот же runtime выполняет → `24`) гоняются через `kenga-lite` / native C, без PyTorch. Большой трансформер Prophet (M5/M6) пока учится numpy-скриптом в `tools/` — это лаборатория, не вход. PyTorch на 70B это не замена.

---

## Язык

`fn` `let` `if`/`else if` `while` `for` `break`/`continue` · i64/f64/str/list/struct · `& | ^ ~ << >>` · `0x`/`0b` · import · events.

Три исполнения одного диалекта: lite C VM, more-VM, bytecode→native (`scripts\bc-run.cmd`). Дыры и таблица: [docs/DIALECT_GAP.md](docs/DIALECT_GAP.md).

```bat
bootstrap\bin\kenga-lite.exe run examples\selfhost\bitops.kenga
bootstrap\bin\kenga-lite.exe run examples\selfhost\hex_lab.kenga
scripts\bc-run.cmd examples\ml\kenga_birth.kenga
```

Полный self-host без gcc/cl — последний шаг лестницы. `kenga_lite.c` — каркас из ~90 строк includes.

---

## LM на языке (не Python)

```bat
bootstrap\bin\kenga-lite.exe run examples\ml\kenga_net.kenga      # XOR MLP
bootstrap\bin\kenga-lite.exe run examples\ml\kenga_lm.kenga       # decoder next-token
scripts\kenga-birth.cmd                                          # пишет программу → 24
bootstrap\bin\kenga-lite.exe chat minds\multi.km                  # векторы, не чат-LLM
```

Это не Grok и не HuggingFace-гигант. Словарь крошечный, D/L маленькие, GPU-ядра нет. Честная таблица:

| Есть | Нет |
|---|---|
| Язык + VM + native C из `.kenga` | Linguist / полоска «Kenga» на github.com |
| Decoder / tape / Prophet API в `.kenga` | 50k vocab, L=32, CUDA |
| Birth → 24 одним рантаймом | Замена PyTorch |

Исследования (Prophet numpy, «как 27B», pico 5/5): [docs/KENGA_LM.md](docs/KENGA_LM.md), [docs/NEUROMODEL_27B.md](docs/NEUROMODEL_27B.md). Не с этого начинать.

---

## Установка

```bash
bootstrap\build.cmd          # Windows
bash bootstrap/build.sh      # Linux / macOS / Git Bash
```

Native exe из своего файла (нужен gcc или cl):

```bat
scripts\bc-run.cmd examples\hello.kenga
```

Пересобрать lite из `.kenga`: `bootstrap\rebuild-from-kenga.cmd`.

Полный CLI `kenga.exe` из [Releases](https://github.com/GermannM3/kenga-lang/releases) — только GPU/legacy `src/`. Друзьям он не нужен.

```
kenga-lite run <file.kenga>
kenga-lite eval '<src>'
kenga-lite chat [mind.km]
```

---

## Дерево

```
kenga/            канон: compiler + emit
bootstrap/        C99 host
examples/         язык + ml + selfhost
editors/vscode/   грамматика (VSIX 3.13.0)
editors/linguist/ черновик для github-linguist
src/              legacy Rust, новый код сюда не идёт
tools/            лабораторный Python (корпус / Prophet), не runtime
```

Учить: [docs/LEARN.md](docs/LEARN.md) · упражнения: [docs/EXERCISES.md](docs/EXERCISES.md) · карта вместо Rust: [docs/REPLACE_RUST.md](docs/REPLACE_RUST.md) · план: [docs/ROADMAP.md](docs/ROADMAP.md).

Книга языка: [book/](book/) — хроника, не научная монография.

## Лицензия

[MIT](LICENSE) © Kenga AI / [GermannM3](https://github.com/GermannM3)
