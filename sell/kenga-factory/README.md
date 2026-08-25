# Kenga Verified Factory

Грамматика + интерпретатор → программы с проверенным stdout.

## Воспроизведение

`
bootstrap\build.cmd
python tools\corpus_eval.py minds\corpus_factory\split_v2\test.jsonl --category bind --limit 144
`

Отчёт: minds/corpus_factory/M6_REPORT.md
Корпус v2: 14550 программ (карточка M5.3).

## Честно

25.08 утро: factory compile 98.3%, bind 96.1%, NL greedy match 1.2%, realgen T1 0% → Genesis закрыт. Спека цикла: docs/GENESIS_V0.md — не запущена.
