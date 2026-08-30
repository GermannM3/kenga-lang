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

В языке же: тензоры, tape-autograd, память `remember` / `foresee`, события. Это библиотека рантайма, не отдельный продукт.

---

## Язык

`fn` `let` `if`/`else if` `while` `for`/`step` `break`/`continue` · i64/f64/str/list/struct · `match`/`enum` · `+=` · `& | ^ ~ << >>` · `0x`/`0b` · `slice`/`index_of`/`starts_with`/`split` · `map_*`/`json_set` · import · events · HTTP/JSON.

Три исполнения одного диалекта: lite C VM, more-VM, bytecode→native (`scripts\bc-run.cmd`). Дыры и таблица: [docs/DIALECT_GAP.md](docs/DIALECT_GAP.md).

```bat
bootstrap\bin\kenga-lite.exe run examples\selfhost\bitops.kenga
bootstrap\bin\kenga-lite.exe run examples\selfhost\hex_lab.kenga
bootstrap\bin\kenga-lite.exe run examples\selfhost\struct_lite.kenga
bootstrap\bin\kenga-lite.exe run examples\selfhost\str_lab.kenga
bootstrap\bin\kenga-lite.exe run examples\selfhost\net_lite.kenga
```

Telegram-бот (токен в env): `examples/telegram_bot.kenga` — файл + Пророк 16-d, учит с чата. VPS: [docs/VPS.md](docs/VPS.md).

Полный self-host без gcc/cl — последний шаг лестницы. `kenga_lite.c` — каркас из ~90 строк includes.

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
examples/         язык, selfhost, упражнения
editors/vscode/   грамматика (VSIX 3.13.0)
src/              legacy Rust, новый код сюда не идёт
```

Учить: [docs/LEARN.md](docs/LEARN.md) · упражнения: [docs/EXERCISES.md](docs/EXERCISES.md) · тур: [docs/TOUR.md](docs/TOUR.md) · карта вместо Rust: [docs/REPLACE_RUST.md](docs/REPLACE_RUST.md).

Книга языка: [book/](book/) — хроника, не научная монография.

## Лицензия

[MIT](LICENSE) © Kenga AI / [GermannM3](https://github.com/GermannM3)
