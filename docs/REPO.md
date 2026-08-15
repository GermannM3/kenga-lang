# Что лежит в репозитории

Репозиторий — для **пользователей и форков**, не свалка артефактов.

## Есть на GitHub

| Путь | Зачем |
|---|---|
| `README.md`, `docs/` | ставить, учить, запускать |
| `examples/` | рабочие программы |
| `kenga/` | **замена Rust**: compiler/emit на `.kenga` |
| `bootstrap/` | C99 runtime (временный host) |
| `editors/vscode/` | расширение + актуальный `.vsix` |
| `minds/*.km` | демо для chat |
| `scripts/` | smoke |
| `src/`, `Cargo.toml` | **legacy** Rust-хост (см. `docs/REPLACE_RUST.md`) |

## Нет на GitHub (gitignore)

- `target/`, бинарники, `bootstrap/bin/`
- emit-артефакты, `tmp_*`
- веса `minds/word_lm_*`, `minds/_*`
- старые `.vsix` (только текущий)

Друзьям: Releases **или** `bootstrap\build.cmd`.  
Свобода: `docs/INDEPENDENCE.md` · замена Rust: `docs/REPLACE_RUST.md`.
