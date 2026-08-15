# Kenga для знакомых (ML)

Коротко, без воды — что сказать друзьям и как не опозориться.

## Одной фразой

Kenga — язык, где **память, surprise и world-model** в семантике, а не в pip-пакетах. Сейчас это рабочий bootstrap: свой синтаксис + VM + Prophet. Python не нужен. Компилятор пока на Rust (как ранний Go на C). Self-host уже в лаборатории (`examples/selfhost/`).

## Что показать за 5 минут

```bash
git clone https://github.com/GermannM3/kenga-lang.git
cd kenga-lang
cargo install --path . --force --locked
kenga about
kenga demo
```

Потом руками:

```bash
kenga run examples/ml/world_model.kenga
kenga chat minds/agent.km
# «смотри 5 1 6» / «что будет завтра?» / «статус»
```

## Честный дисклеймер (обязательно)

| Да | Нет |
|---|---|
| Свой язык + VM | Не LLM-чат из коробки |
| Living memory + MLP world-model | Не замена PyTorch/JAX |
| Агентный event loop | Не production CUDA stack |
| Self-host ladder на Kenga | Полный self-host ещё не закрыт |
| MIT, открытый репо | Не «AGI в выходные» |

World-model предсказывает динамику векторов состояний (`[pos,vel,fuel]→next`), а не пишет эссе.

## Зачем это ML-щику смотреть

1. **Другая ось**, не «ещё один фреймворк на Python».
2. Можно потрогать **surprise-gated memory** и unroll будущего без зоопарка зависимостей.
3. Self-host путь открыт: Kenga-lite уже компилируется в bytecode, написанный на Kenga.

## Если сломалось

- `kenga version` → должно быть **2.8.x**
- Linux/Mac/Git Bash: `docs/UNIX.md` · `bash scripts/unix-smoke.sh`
- `unknown command chat` → старый бинарник: `cargo install --path . --force`
- `kenga-lite not built` → `bootstrap\build.cmd` (нужен MSVC/gcc)
- нет `examples/` → запускай из корня клона
- `kenga which` — какой exe в PATH

## Иконки `.kenga` в Cursor / VS Code

Без расширения файлы выглядят как «голый текст». Поставь локальное расширение из репо:

```powershell
.\editors\install-extension.cmd
# или: cursor --install-extension .\editors\vscode\kenga-2.3.0.vsix
```

Reload Window — у `.kenga` появится свой значок (K) и подсветка.

## Ссылки

- Репо: https://github.com/GermannM3/kenga-lang  
- Язык: `docs/LANGUAGE.md`  
- Self-host / Rust-free: `docs/SELFHOST.md`  
- Roadmap: `docs/ROADMAP.md`  
- Exercises: `docs/EXERCISES.md`  
- Bootstrap: `docs/BOOTSTRAP.md`  
- Editor: `editors/vscode/`
