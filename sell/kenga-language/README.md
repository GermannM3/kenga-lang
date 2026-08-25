# Kenga Language Stack

Язык + компилятор на себе + VM + emit C99. Lite — один exe без pip/cargo.

## Воспроизведение

`
git clone https://github.com/GermannM3/kenga-lang.git
cd kenga-lang
bootstrap\build.cmd
bootstrap\bin\kenga-lite.exe run examples\selfhost\fact_lite.kenga
scripts\freedom-smoke.cmd
`

freedom-smoke 25.08.2026 — OK.

## Документы

docs/SPEC.md, docs/TOUR.md, docs/INDEPENDENCE.md, docs/FOR_FRIENDS.md, docs/REPLACE_RUST.md

## Честно

Self-host без C host не закрыт. src/ Rust — legacy. Не LLVM, не CUDA.

Грамматика для GitHub Linguist: https://github.com/GermannM3/kenga-grammar
PR в Linguist не открываем, пока мало чужих `.kenga`.
