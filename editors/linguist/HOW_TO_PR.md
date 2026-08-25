# Linguist PR (не Marketplace)

На github.com язык, полоска и подсветка `.kenga` идут из [github-linguist/linguist](https://github.com/github-linguist/linguist), не из VS Code / VSIX 3.13.0.

Пока PR не влит и Linguist не выпущен — GitHub рисует `.kenga` как plain text. `.gitattributes` в этом репо только готовит статистику (generated/vendored).

## Что нужно Linguist

Источник: [CONTRIBUTING.md](https://github.com/github-linguist/linguist/blob/main/CONTRIBUTING.md).

1. Запись в `lib/linguist/languages.yml` — сниппет в `languages.yml` рядом (без `language_id`).
2. Грамматика: `script/add-grammar https://github.com/GermannM3/kenga-lang`
   TextMate: `editors/vscode/syntaxes/kenga.tmLanguage.json`, `scopeName: source.kenga`, лицензия MIT (в allowlist).
   Если скрипт откажется от монорепы/`$schema` — вынести grammar в отдельный public repo и дать его URL.
3. Сэмплы в `samples/Kenga/`. **Hello world и уроки не принимают.** Не класть `examples/hello.kenga` / `examples/tour/01_hello.kenga`.
   Для PR: `kenga/emit/*.kenga`, `examples/bitops.kenga`, `examples/selfhost/bc_mem.kenga`, `examples/selfhost/struct_lite.kenga`.
   Крошечные копии лежат в `samples/` (можно отдать Linguist под MIT).
4. `script/update-ids`.
5. PR **только по шаблону Linguist**. В теле: лицензия сэмплов (MIT) + поиск usage.

`.kenga` в Linguist сейчас нет, эвристика не нужна.

## Порог (главный блокер)

Не хобби-языки. GitHub Search, **не форки**, файлы за последний год:

- расширение вроде `.kenga`: **≥ 2000 файлов**
- и размазано по разным `:user/:repo`; основного автора вычтут (`-user:GermannM3`)

Запрос: <https://github.com/search?type=code&q=NOT+is%3Afork+path%3A*.kenga>

Пока счётчик маленький или всё из одного аккаунта — PR закроют. Не открывать.

После мержа github.com подхватит язык на **релизе Linguist**, не в тот же день ([troubleshooting](https://github.com/github-linguist/linguist/blob/main/docs/troubleshooting.md)).
