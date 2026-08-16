# Publishing the VS Code / Cursor extension

Publisher id in `editors/vscode/package.json`: **`Kenga-ai`**.  
На маркетплейсе **3.13.0** (подсветка + иконка `.kenga`).  
Язык в git уже дальше: `more.kenga` гоняет Prophet/tape/ppm-wav/`learn`/`unroll`/living multimodal, `lower_kv` эмитит Memory/Tensor/tape, `lower_c`/`bc_src_c` — `typeof`/`print`/`sleep_ms`/`now_ms`.  
Грамматика расширения эти слова уже знает — **версию VSIX не поднимаем**, пока не меняется editor. Следующий `.vsix`, когда накопится в `editors/vscode`.

## Ручная заливка (без Azure / без vsce login)

Если PAT/Azure DevOps недоступны (нет карты, регион и т.п.) — этого достаточно:

1. Собери VSIX:

```bat
cd editors\vscode
npx --yes @vscode/vsce package --skip-license -o kenga-3.13.0.vsix
```

2. Открой https://marketplace.visualstudio.com/manage/publishers/Kenga-ai  
3. Extension → **Update** → загрузи только `.vsix` (не `package.json`).

Или локально: `.\editors\install-extension.cmd`

## Опционально: vsce + PAT

Нужен Azure DevOps org + Personal Access Token со scope **Marketplace**.  
Платная Azure-подписка для токена обычно не нужна — но если org/карта блокируются, используй ручную заливку выше.

```bash
cd editors/vscode
npm i -g @vscode/vsce   # или всегда через npx
npx @vscode/vsce login Kenga-ai
npx @vscode/vsce publish
```

## Open VSX

```bash
npx ovsx publish -p <OPEN_VSX_TOKEN>
```

https://open-vsx.org/

## Hugging Face (модель, не расширение)

Когда большая мультимодальная LM готова — git (пример) и HF. Сейчас сид и упаковка: `docs/HUGGINGFACE.md`.

