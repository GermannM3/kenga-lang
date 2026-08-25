# Kenga Verified Factory - письмо

Одна просьба во всех трёх языках: 30 минут с человеком, у кого eval или данные.
Genesis не продаём. Realgen T1 compile 0% говорим сами.

---

## RU

**Тема:** Проверенный корпус из интерпретатора, который у вас уже есть

Пишу, потому что вы, скорее всего, учите или меряете модель на языке, которого нет на GitHub. Внутренний SQL-диалект, конфиг-DSL, контракты, инструменты агента: качать нечего. Людей на разметку программ не напасёшься, а дриллы из большой модели врут, пока их не прогнал ваш рантайм.

Мы сделали фабрику. На входе грамматика и интерпретатор, которые у вас уже крутятся. На выходе программы, которые скомпилировались и напечатали ожидаемый stdout. Пары broken/fixed идут из той же трубы: ломаем проверенную программу, оставляем обе стороны. Часов человеческой разметки: ноль. Генератор: `tools/corpus_factory.py`. Он зовёт настоящий kenga-lite, не заглушку. Рядом лежат `tools/corpus_eval.py`, `tools/realgen_eval.py`, `tools/build_repair_corpus.py`.

На нашем языке корпус v2: 14 550 проверенных программ, семейства arith, loop, rec, chains, bind. На встрече я хочу говорить про bind. Та же модель примерно 838k параметров, тот же бюджет обучения. Пока в распределении жил статистический ярлык (main всегда звал одно и то же имя), bind compile был 5,6%. Ярлык убрали. Стало 98,6%. Цифры: `hf/Kenga-Prophet-m5.3/README.md`.

Притворяться, что модель пишет настоящий код, я не буду. Утренний отчёт M6 от 25 августа 2026: factory compile 98,3%, greedy match 18,3%; bind compile 96,1%; NL->code compile 50%, greedy match 1,2%; realgen T1 compile 0%. Цикл Genesis в `docs/GENESIS_V0.md` это спецификация, не продукт. Ворота закрыты на realgen. Если говорить про партнёрство, то чтобы сдвинуть этот ноль, а не чтобы я вам сказал, что цикл уже работает.

Три двери, если разговор пойдёт дальше: пилот на 4 недели (ваша грамматика и интерпретатор в проверенный корпус), покупка тулинга, исследование обучения под проверятором. Сейчас одна просьба. Тридцать минут с тем, кто у вас отвечает за eval или данные. В колоде две команды, которыми воспроизводится bind-eval. Вы приносите язык, который нельзя наскрести.

---

## EN

**Subject:** A verified corpus from the interpreter you already run

I'm writing because you train or evaluate models on a language with no public dump. Internal SQL dialect, config DSL, contracts, agent tools: nothing to scrape. Paying people to label programs does not scale, and LLM drills still lie until your runtime runs them.

We built a factory. Input: a grammar and an interpreter you already run. Output: programs that compiled and printed the expected stdout. Repair pairs come from the same pipe: mutate a verified program, keep broken and fixed. Human labeling hours: zero. Generator: `tools/corpus_factory.py`, calling real kenga-lite, not a mock. Also on disk: `tools/corpus_eval.py`, `tools/realgen_eval.py`, `tools/build_repair_corpus.py`.

On our language, corpus v2 is 14,550 verified programs: arith, loop, rec, chains, bind. Bind is why I want the meeting. Same ~838k model, same training budget. When the distribution still had a statistical shortcut (main always called the same name), bind compile was 5.6%. Shortcut gone: 98.6%. Table: `hf/Kenga-Prophet-m5.3/README.md`.

I will not dress this as the model writing real programs. M6 report 2026-08-25 (`minds/corpus_factory/M6_REPORT.md`): factory compile 98.3%, greedy match 18.3%; bind compile 96.1%; NL to code compile 50%, greedy match 1.2%; realgen T1 compile 0%. Genesis in `docs/GENESIS_V0.md` is a spec, not a product. Gate closed on realgen. Partnership would be to move that zero, not to pretend the loop is running.

One ask: thirty minutes with whoever owns eval or data. Two commands to reproduce the bind eval are in the deck. You bring the language you cannot scrape.

---

## ZH

**主题:** 用你们已经在跑的解释器，做出已验证的程序语料

我写信，是因为你们多半在一门没有公开语料的语言上训练或评测模型。内部 SQL 方言、配置 DSL、合约、agent 工具：网上刮不到。雇人标程序不划算，大模型吐出来的练习题，不经你们自己的运行时，照样能编。

我们做了一个工厂。输入是你们已经有的文法和解释器。输出是真正编译过、stdout 对得上预期的程序。同一条管道再吐 broken/fixed：把已验证的程序弄坏，两边都留下。人工标注小时数：零。生成器是 `tools/corpus_factory.py`，调的是真的 kenga-lite，不是 mock。旁边还有 `tools/corpus_eval.py`、`tools/realgen_eval.py`、`tools/build_repair_corpus.py`。

我们自己这门语言上，v2 语料是 14,550 条已验证程序，五个家族：arith、loop、rec、chains、bind。我想拿出来谈的是 bind。同一套大约 838k 的模型，同一份训练预算。训练分布里还留着统计捷径时（main 总是调同一个名字），bind 编译率是 5.6%。捷径拿掉以后，98.6%。数字在 `hf/Kenga-Prophet-m5.3/README.md`。

我不会把这说成"模型会写真实程序"。2026-08-25 的 M6 早报（`minds/corpus_factory/M6_REPORT.md`）：factory 编译 98.3%，greedy match 18.3%；bind 编译 96.1%；NL->code 编译 50%，greedy match 1.2%；realgen T1 编译 0%。`docs/GENESIS_V0.md` 里的 Genesis 循环是规格，不是在跑的产品。入口卡在 realgen 上，关着。如果谈合作，是为了把这个 0% 挪动，不是声称循环已经能用。

后面若能往下谈，有三扇门：四周试点（你们的文法加解释器，换一份已验证语料）、买这套工具、做验证器把关的训练研究。现在只求一件事：和你们的评测或数据负责人聊 30 分钟。deck 里有两条可复现 bind 评测的命令。你们带上那门刮不到的语言就行。
