# Сопроводительное письмо — Z-паспорт весов

## Русский
**Тема:** M6 с паспортом: 52 тензора, marker 6108e4d16400d5e1

Это не презентация модели. Это слой, который отвечает: файл совпал с актом или его подменили. На mid_prophet_m6_w.txt висит passport.json: sha256, marker 6108e4d16400d5e1, 52 тензора, k=32 (сингулярные, не контекст). Контекст M6 — K=512, параметров 887 168.

Паспорт не лечит realgen. Утром 25.08 Genesis закрыт по T1 compile 0%. Двухсидовый контроль для «траектория vs данные» не сделан — этой фразы в письме нет как утверждения.

Проверка: python tools/zcert.py verify ... --cert ...passport.json

Три pptx в папке.

Просьба: 30 минут с supply-chain или eval. На встрече гоняем verify, читаем marker вслух.

Герман
https://github.com/GermannM3/kenga-lang

## English
**Subject:** M6 with a passport: 52 tensors, marker 6108e4d16400d5e1

This is not a model pitch. It answers: did the file match the act, or did someone swap it. mid_prophet_m6_w.txt has passport.json: sha256, marker 6108e4d16400d5e1, 52 tensors, k=32 (singular values, not context). M6 context is K=512, 887,168 params.

A passport does not fix realgen. 25 Aug morning: Genesis closed on T1 compile 0%. Two-seed control for trajectory-vs-data is not done, so that sentence is not a claim here.

Check: python tools/zcert.py verify ... --cert ...passport.json

Three decks in the folder.

Ask: 30 minutes with supply-chain or eval. We run verify and read the marker out loud.

Hermann
https://github.com/GermannM3/kenga-lang

## 中文
**主题：** 带护照的 M6：52 个张量，marker 6108e4d16400d5e1

这不是模型推介。它回答：文件和文书是否一致，有没有被换。mid_prophet_m6_w.txt 上有 passport.json：sha256、marker 6108e4d16400d5e1、52 个张量、k=32（奇异值个数，不是上下文）。M6 上下文是 K=512，参数 887168。

护照治不好 realgen。8月25日上午 Genesis 因 T1 compile 0% 关闭。轨迹对数据的双种子对照没做，所以信里不当结论写。

核对：python tools/zcert.py verify ... --cert ...passport.json

文件夹里三份 pptx。

请求：和供应链或评测谈 30 分钟。会上跑 verify，把 marker 读出来。

Hermann
https://github.com/GermannM3/kenga-lang
