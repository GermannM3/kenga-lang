# Linux / macOS / Git Bash

PowerShell ≠ bash. В bash `#` — комментарий **до конца строки**, а вставка
целого блока из чата иногда глотает следующие команды. Запускай **по одной**.

## PATH

```bash
export PATH="$HOME/.cargo/bin:$PATH"
kenga version
```

Git Bash на Windows: `~/.cargo/bin` → `/c/Users/<you>/.cargo/bin`.

Положи в `~/.bashrc`:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

## Без установки Rust (друзья)

1. Скачай архив с [Releases](https://github.com/GermannM3/kenga-lang/releases)  
   (`kenga-linux-x86_64.tar.gz` / `kenga-macos-arm64.tar.gz` / …).
2. `chmod +x kenga kenga-lite` и положи в `PATH` или в корень клона.
3. `git clone` репо (нужны `examples/`).
4. `kenga demo`

Lite без Rust вообще (только C-компилятор):

```bash
bash bootstrap/build.sh
./bootstrap/bin/kenga-lite run examples/hello.kenga
./bootstrap/bin/kenga-lite run examples/native_lists.kenga
kenga run --lite examples/selfhost/for_lite.kenga
```

macOS: если нет `cc` → `xcode-select --install`.

Chicken-egg (нужен `kenga` один раз для emit-c):

```bash
bash bootstrap/rebuild-from-kenga.sh
```

## Smoke

```bash
bash scripts/unix-smoke.sh
```

## Что ещё на Rust-хосте

Полный язык (Memory, Tensor, `ag_*`, events) — бинарник `kenga` из Releases
или `cargo install`. Lite растёт: уже f64, else if, assert, type annotations,
stubs `tensor`/`sweep`. Дорога: `docs/ROADMAP.md`.
