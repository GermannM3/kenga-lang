# Kenga for VS Code / Cursor

Версия **3.13.0**. Язык `.kenga` в проводнике получает свой значок (бирюзовая K), подсветка синтаксиса и базовый language config.

Подсветка знает `read_file` / `write_file` / `save_tensor` / `load_tensor` / `ag_*` / `t_softmax` — то, чем живут birth и mm-LM.

## Установка из этого репо

```powershell
.\editors\install-extension.cmd
```

Скрипт ставит VSIX и кладёт SVG для Material Icon Theme. После **Reload Window** у `.kenga` свой значок.

Если иконка всё ещё «текстовый лист» (Material Icon Theme), в **User Settings**:

```json
"material-icon-theme.files.associations": {
  "*.kenga": "../../icons/kenga"
}
```

## Dev Host (без install)

Command Palette → **Developer: Install Extension from Location…** → выбери `editors/vscode`.

## Что внутри

- language id `kenga`, расширение `.kenga`
- file icon (light/dark) из бренда Kenga
- TextMate grammar: keywords, types, builtins, strings, comments
- brackets / comments configuration
