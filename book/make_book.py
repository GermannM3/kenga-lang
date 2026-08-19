# -*- coding: utf-8 -*-
"""
КЕНГА — книга Германа Янтараса.
A5 PDF в семействе Z-системы + EPUB из того же текста.
"""
import html
import os
import re
import sys

import matplotlib
from ebooklib import epub
from reportlab.lib.enums import TA_CENTER, TA_JUSTIFY
from reportlab.lib.pagesizes import A5
from reportlab.lib.styles import ParagraphStyle
from reportlab.lib.units import cm
from reportlab.pdfbase import pdfmetrics
from reportlab.pdfbase.ttfonts import TTFont
from reportlab.platypus import (
    BaseDocTemplate, Frame, Image, NextPageTemplate, PageBreak,
    PageTemplate, Paragraph, Preformatted, Spacer, Table, TableStyle,
)
from reportlab.platypus.tableofcontents import TableOfContents

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
PDF_OUT = os.path.join(HERE, "kenga_kniga_yantaras_v1.pdf")
EPUB_OUT = os.path.join(HERE, "kenga_kniga_yantaras_v1.epub")
COVER = os.path.join(HERE, "cover.png")
BACK = os.path.join(HERE, "back.png")

FONTS = os.path.join(matplotlib.get_data_path(), "fonts", "ttf")
pdfmetrics.registerFont(TTFont("DJV", os.path.join(FONTS, "DejaVuSans.ttf")))
pdfmetrics.registerFont(TTFont("DJVB", os.path.join(FONTS, "DejaVuSans-Bold.ttf")))
pdfmetrics.registerFont(TTFont("DJVI", os.path.join(FONTS, "DejaVuSans-Oblique.ttf")))
pdfmetrics.registerFont(TTFont("MONO", os.path.join(FONTS, "DejaVuSansMono.ttf")))

W, H = A5
M = 1.5 * cm

st_body = ParagraphStyle("body", fontName="DJV", fontSize=9, leading=12.6,
                         alignment=TA_JUSTIFY, spaceAfter=4)
st_part = ParagraphStyle("part", fontName="DJVB", fontSize=17, leading=22,
                         alignment=TA_CENTER, textColor="#1a3a6b",
                         spaceBefore=6, spaceAfter=10)
st_h1 = ParagraphStyle("h1", fontName="DJVB", fontSize=12.5, leading=16,
                       spaceBefore=12, spaceAfter=6, textColor="#1a3a6b")
st_h2 = ParagraphStyle("h2", fontName="DJVB", fontSize=10.5, leading=13,
                       spaceBefore=8, spaceAfter=4, textColor="#2a5aa8")
st_cap = ParagraphStyle("cap", fontName="DJV", fontSize=7.5, leading=9.5,
                        alignment=TA_CENTER, textColor="#666666",
                        spaceBefore=2, spaceAfter=8)
st_quote = ParagraphStyle("q", fontName="DJVI", fontSize=9, leading=12.5,
                          leftIndent=18, rightIndent=10, textColor="#444444",
                          spaceBefore=4, spaceAfter=6)
st_mono = ParagraphStyle("mono", fontName="MONO", fontSize=6.8, leading=8.4)
st_tbl = ParagraphStyle("tbl", fontName="DJV", fontSize=7.6, leading=9.5)
st_tblh = ParagraphStyle("tblh", fontName="DJVB", fontSize=7.6, leading=9.5)
st_center = ParagraphStyle("ctr", fontName="DJV", fontSize=9, leading=12,
                           alignment=TA_CENTER, textColor="#555555")

PARTS, CHAPTERS = [1], [0]
BLOCKS = []


def roman(n):
    vals = ["I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X"]
    return vals[n - 1] if 1 <= n <= len(vals) else str(n)


def add(kind, **kw):
    BLOCKS.append({"kind": kind, **kw})


def P(t):
    add("p", text=t)


def PART(t):
    add("part", text=t)


def CH(t):
    add("ch", text=t)


def H2(t):
    add("h2", text=t)


def Q(t):
    add("q", text=t)


def TBL(headers, rows, widths=None):
    add("tbl", headers=headers, rows=rows, widths=widths)


def PRE(src, title=None):
    add("pre", text=src, title=title)


def _TOC_ST(level):
    return ParagraphStyle(
        f"toc{level}", fontName="DJVB" if level == 0 else "DJV",
        fontSize=9.5 if level == 0 else 9, leading=13 if level == 0 else 12,
        leftIndent=0 if level == 0 else 14, rightIndent=24,
        textColor="#1a3a6b" if level == 0 else "#333333")


class Book(BaseDocTemplate):
    def afterFlowable(self, fl):
        toc = getattr(fl, "_toc", None)
        if toc:
            self.notify("TOCEntry", (0, toc, self.page))


def on_page(canv, doc):
    canv.saveState()
    canv.setFont("DJV", 7.5)
    canv.setFillColor("#888888")
    canv.drawCentredString(W / 2, 0.85 * cm, str(canv.getPageNumber()))
    canv.setFont("DJVI", 7)
    canv.drawString(M, H - 1.1 * cm, "Кенга · Герман Янтарас")
    canv.setStrokeColor("#cccccc")
    canv.setLineWidth(0.4)
    canv.line(M, H - 1.25 * cm, W - M, H - 1.25 * cm)
    canv.restoreState()


def write_content():
    BLOCKS.clear()
    PARTS[0] = 1
    CHAPTERS[0] = 0

    add("front")

    P("© Герман Янтарас, 2026")
    P("Эта книга описывает язык Kenga таким, каким он есть в репозитории "
      "kenga-lang на момент первого издания: версия хоста 3.13, канон без "
      "Rust в каталогах bootstrap/ и kenga/. Все команды воспроизводимы. "
      "Если эксперимент не повторяется — виноват текст, не «дух языка».")
    P("Книга написана человеком. Соавтор по коду — тот же рабочий контур, "
      "что собирает kenga-lite. Это не рецензия индустрии и не обещание "
      "модели уровня Grok.")
    add("pagebreak")

    add("toc")

    add("intro", text="Введение. Язык, который не врёт про себя")
    P("Kenga — язык программирования для живого ИИ: память, предсказание, "
      "агентный цикл и тензор живут в семантике, а не в обёртке над чужим "
      "фреймворком. Это коротко. Дальше начинается то, ради чего стоит "
      "держать книгу рядом с репозиторием.")
    P("Обычный путь «языка для ИИ» выглядит так: синтаксис поверх Python, "
      "веса скачаны с Hugging Face, доказательство — чужой GGUF в папке с "
      "вашим логотипом. Мы этот путь откладываем. Доказательство Kenga — "
      "сеть, написанная на Kenga и обученная на Kenga. Пока сеть крошечная. "
      "Это не стыдно, если не называть её половиной чужой модели.")
    P("Вторая нить книги — свобода от Rust. Хост kenga-lite собирается "
      "C99-компилятором. Файл bootstrap/kenga_lite.c почти пустой: CRT и "
      "цепочка #include \"generated/rt_*.inc.c\". Сами rt_* пишет Kenga. "
      "Компилятор more.kenga уже запускает чужие программы на своём VM, "
      "в том числе ту, которую написала suffix-модель: fact(add(2,3)-1) "
      "печатает 24.")
    Q("Скачанный GGUF не доказывает язык. Доказывает архитектура, обучение "
      "и корпус в .kenga.")
    P("Структура. Часть I — диалект, чтобы открыть файл и не утонуть. "
      "Часть II — зачем уходить с Rust и как устроен каркас. Часть III — "
      "компилятор на Kenga и петля birth. Часть IV — память, тензор, XOR. "
      "Часть V — своя языковая модель и честная таблица того, чего нет. "
      "Часть VI — практикум и лабораторный журнал. Приложение — карта "
      "файлов и встроенные функции.")
    P("Тон тот же, что в книге по Z-системе: аксиома без эксперимента — "
      "декор. Отрицательный результат оставляем в тексте.")

    PART("Язык")
    CH("Что такое Kenga")
    P("Kenga — императивный язык с функциями, списками, структурами, "
      "событиями и встроенным тензором. Хост 3.13 умеет два пути: полный "
      "бинарник с Releases (ещё содержит Rust) и kenga-lite из bootstrap/, "
      "который Rust не вызывает.")
    P("<b>F1.</b> Живой runtime без Rust существует: Prophet Memory, "
      "тензор, tape, события, компилятор и VM собираются из эмиттеров в "
      "kenga/emit/.")
    P("<b>F2.</b> Расширение VS Code на маркетплейсе — 3.13.0. Язык уже "
      "дальше (word-decoder, rt_* хост). Номер расширения не двигаем, "
      "пока не меняется сам редактор.")
    H2("Чем язык не является")
    P("Не ChatGPT из коробки. Не замена PyTorch. Не pretrained CLIP или "
      "Whisper. Не Grok и не чужой GGUF под другим именем. Сид для "
      "Hugging Face лежит в hf/kenga-seed/; большую карточку "
      "Kenga-ai/kenga-mm туда класть рано.")
    TBL(["Да", "Нет"], [
        ["Свой язык, VM, living memory", "Готовый чат-бот"],
        ["Decoder в .kenga, крошечный корпус", "Словарь 50k и контекст 4k"],
        ["birth → программа → 24", "Доказательство через GGUF"],
        ["Kenga пишет C99 (c_seed, lower_c)", "VM без C-хоста"],
    ], [6.2 * cm, 6.2 * cm])
    P("Если знакомый спрашивает «это как Python для нейросетей?» — нет. "
      "Python клеит C++/CUDA. Kenga держит тензор и агентный цикл как "
      "часть языка. Масштаб пока учебный. Это разные утверждения, и оба "
      "верны одновременно.")

    CH("Установка без Rust")
    P("Друзьям проще начать с бинарника. Клонируешь репозиторий — нужны "
      "examples/ — кладёшь kenga в PATH, запускаешь demo.")
    PRE(
        "git clone https://github.com/GermannM3/kenga-lang.git\n"
        "cd kenga-lang\n"
        "bootstrap\\build.cmd\n"
        "bootstrap\\bin\\kenga-lite.exe version",
        "Сборка lite")
    P("build.cmd гоняет selftest: 52 случая. Если падает — чини хост, "
      "не книгу. Дымовой контур свободы: scripts\\freedom-smoke.cmd.")
    P("Полный cargo install остаётся для GPU и legacy-пути. Новый код "
      "в src/ не кладём. Карта замены: docs/REPLACE_RUST.md. Путь к "
      "самостоятельности: docs/INDEPENDENCE.md.")
    H2("Первый запуск")
    PRE(
        "fn main() -> i64 {\n"
        "    println(\"hello from kenga\");\n"
        "    let x: i64 = 21;\n"
        "    let y: i64 = 2;\n"
        "    println(x * y);\n"
        "    return 0;\n"
        "}",
        "examples/hello.kenga")
    P("Запуск: kenga run --lite examples/hello.kenga — или тот же файл "
      "через kenga-lite.exe run. Печатает приветствие и 42.")
    P("Дальше по вечеру: docs/LEARN.md. Справочник синтаксиса: "
      "docs/LANGUAGE.md. Упражнения без подсказок из examples/: "
      "docs/EXERCISES.md.")

    CH("Диалект")
    P("Lite-диалект уже покрывает то, без чего не напишешь компилятор и "
      "маленькую сеть: целые, f64, строки, списки, struct, if / else if, "
      "while, for, break, continue, вызовы функций вперёд по тексту.")
    H2("Условия и циклы")
    P("if cond { … } else if … else { … }. while cond { … }. "
      "for i in 0..n и for x in xs. Смотри examples/showcase.kenga и "
      "examples/selfhost/for_lite.kenga.")
    H2("Списки и структуры")
    PRE(
        "let xs = [1, 2, 3];\n"
        "xs = push(xs, 4);\n"
        "println(len(xs));\n"
        "\n"
        "struct Point { x, y }\n"
        "let p = Point { x: 3, y: 4 };\n"
        "println(p.x);",
        "список и struct")
    P("Шпаргалки: examples/selfhost/lists_lite.kenga, struct_lite.kenga. "
      "lower_c.kenga уже умеет опустить struct в C.")
    H2("Ограничения, о которые бьёшься руками")
    P("В lite нет && и || — пиши вложенный if. Подчёркивание в именах "
      "можно. Не называй свою функцию predict: это встроенная. Не гони "
      "strip_line_comments по огромному файлу перед take — intern там "
      "квадратичный, зависает незаметно.")
    P("Строки в more.kenga долго ломались на экранах. Старый parse_string "
      "считал \\\" концом строки. Из-за этого c_seed.kenga раздувался до "
      "сотен мегабайт и его убивали. Когда экраны починили, more стал "
      "прогонять c_seed и expr_c и писать .c. Это не «красивый рефакторинг». "
      "Это баг, без которого петля emit не закрывается.")

    CH("События и агент")
    P("Агент в Kenga — не фреймворк. Это on / emit / pump / pending. "
      "Обработчик подписывается на имя события, emit кладёт сообщение в "
      "очередь, pump её разбирает.")
    PRE(
        "on \"tick\"(n) {\n"
        "    println(n);\n"
        "}\n"
        "emit(\"tick\", 1);\n"
        "pump();",
        "схема событий")
    P("Живой пример: examples/agent.kenga. На lite: "
      "kenga run --lite examples\\agent.kenga. Тот же цикл уже есть в "
      "more.kenga — компилятор на Kenga гоняет агента без отдельного C-VM.")
    P("Chat-интенты (kenga chat --lite minds/agent.km) сидят в "
      "kenga/emit/rt_chat.kenga. Это не языковая модель. Это разбор фраз "
      "в духе «смотри 5 1 6» и «что будет завтра?» поверх Prophet.")

    PART("Свобода")
    CH("Почему не Rust")
    P("Rust-хост в src/ умеет больше и держит GPU-путь. Он же якорь: "
      "пока новый смысл пишется на Rust, язык не живёт на себе. Правило "
      "репозитория простое. Новый код — в kenga/ или bootstrap/. src/ "
      "не расширяем. Когда Releases станут lite-only, src/ уйдёт в архив.")
    P("<b>F3.</b> C и Rust — подмости. Цель — VM на Kenga, потом убрать C.")
    TBL(["Слой", "Без Rust", "Без C"], [
        ["Living runtime (rt_*)", "да", "нет, gcc/cl"],
        ["more.kenga: birth, XOR, c_seed", "да", "нет"],
        ["lower_c / lower_kv", "да", "нет, линкер"],
        ["**bc_src_c** (.kenga → native exe)", "да", "нет, gcc/cl"],
        ["Полный CLI src/", "ещё legacy", "—"],
    ], [5.2 * cm, 3.6 * cm, 3.6 * cm])
    P("Лестница из docs/INDEPENDENCE.md. Сначала lite-хост. Потом "
      "kenga/compiler вытесняет compiler.rs и кусок VM. Потом lower_* "
      "вытесняет codegen.rs. Потом bc_src_c: разбор .kenga в байткод и "
      "сгенерированную C-VM. Дальше — VM на Kenga без C-хоста. Releases "
      "без cargo.")
    Q("kenga_lite.c должен оставаться каркасом includes. Новую логику "
      "хоста руками туда не пишем — эмитим из .kenga.")

    CH("Каркас kenga-lite")
    P("После эмиттеров bootstrap/kenga_lite.c — около девяноста строк: "
      "заголовки CRT и #include generated/rt_*.inc.c. Компилятор, VM, "
      "selftest, Prophet, тензор, tape, события, chat, типы и опкоды "
      "приходят из Kenga.")
    P("Эмиттеры лежат в kenga/emit/. Имена честные: rt_lex, rt_parse, "
      "rt_expr, rt_factor, rt_stmt, rt_compile, rt_vm, rt_selftest, "
      "rt_prophet, rt_tensor, rt_tape, rt_events, rt_chat. Пересобрал "
      "эмиттер — пересобрал host.")
    PRE(
        "bootstrap\\build.cmd\n"
        "scripts\\freedom-smoke.cmd\n"
        "bootstrap\\bin\\kenga-lite.exe run kenga\\compiler\\more.kenga",
        "три команды каркаса")
    P("Старые ручные файлы bootstrap/prophet_lite.inc.c и соседи удалены, "
      "когда соответствующие rt_* стали источником. Если правишь host и "
      "тянешься к .c вместо .kenga — ты идёшь назад по лестнице.")

    CH("Карта kenga/")
    P("Каталог kenga/ — каноническая замена src/*.rs. Не зеркало и не "
      "прокладка. Сюда кладём то, что должно остаться, когда Rust уйдёт.")
    TBL(["Путь", "Роль"], [
        ["compiler/lite.kenga", "узкий compiler+VM на i64"],
        ["compiler/more.kenga", "birth→24, XOR, c_seed/expr_c"],
        ["emit/c_seed.kenga", "Kenga пишет .c"],
        ["emit/expr_c.kenga", "выражение → C99 + проверка"],
        ["emit/lower_c.kenga", "while/if/for/struct/f64 → C"],
        ["emit/rt_*.kenga", "куски runtime → generated/*.inc.c"],
    ], [4.8 * cm, 7.6 * cm])
    P("Полная таблица — в kenga/README.md и docs/REPLACE_RUST.md. Эта "
      "глава не дублирует каждую строку. Смысл один: если модуль ещё "
      "живёт только в Rust, у него должно быть имя файла в kenga/, куда "
      "он переедет.")

    PART("Компилятор на себе")
    CH("more.kenga")
    P("more.kenga — компилятор и VM, написанные на Kenga. Он читает "
      "исходник, собирает байткод, исполняет. На нём уже ходят fact_lite, "
      "kenga_net.kenga (XOR), kenga_birth.kenga и kenga_born.kenga.")
    P("<b>F4.</b> more запускает examples/ml/kenga_born.kenga и печатает "
      "24. Отдельный C-VM для этой проверки не нужен.")
    P("Опкоды read_file / write_file — 32 и 33. Без них birth не может "
      "положить программу на диск, а c_seed не может выписать .c.")
    P("Экраны в parse_string: \\n, \\t, \\\", \\\\. Пока их не было, "
      "emit-файлы с кавычками внутри строк вешали парсер. После правки "
      "more прогоняет kenga/emit/c_seed.kenga и expr_c.kenga.")
    H2("Что more ещё не умеет")
    P("Тензорные опкоды decoder'а на VM more пока не живут. Триграмма на "
      "more может быть слишком медленной, чтобы считать её повседневным "
      "тестом. Это дыры, не «скоро». Пока decoder гоняет kenga-lite, не "
      "more.")

    CH("Петля birth")
    P("kenga_birth.kenga — suffix language model. Она дописывает seed с "
      "fn add, кладёт examples/ml/kenga_born.kenga, и уже другой запуск "
      "исполняет написанное. Ожидаемый вывод — 24: факториал от "
      "add(2,3)-1.")
    PRE(
        "fn add(a: i64, b: i64) -> i64 {\n"
        "  return a + b;\n"
        "}\n"
        "fn fact(n: i64) -> i64 {\n"
        "  let p = 1;\n"
        "  let i = 1;\n"
        "  while i <= n {\n"
        "    p = p * i;\n"
        "    i = i + 1;\n"
        "  }\n"
        "  return p;\n"
        "}\n"
        "fn main() -> i64 {\n"
        "  let x = add(2, 3);\n"
        "  let y = sub(x, 1);\n"
        "  if y > 0 { println(fact(y)); } else { println(0); }\n"
        "  return 0;\n"
        "}",
        "то, что пишет модель (сокращено)")
    P("Команды. scripts\\kenga-birth.cmd — привычный контур. "
      "scripts\\bc-run.cmd examples\\ml\\kenga_birth.kenga — birth через "
      "native C без lite VM, потом kenga-lite run examples\\ml\\kenga_born.kenga. "
      "Или сразу more: он сам пишет born и исполняет.")
    P("<b>F5.</b> Birth — не GGUF. Это suffix-LM на корпусе языка. "
      "Программа, которую она пишет, короткая и заранее известная по "
      "форме. Ценность не в «творчестве», а в замкнутом круге: модель на "
      "Kenga производит Kenga, хост на Kenga это исполняет.")
    Q("24 — не магия. Это fact(4). Магия была бы, если бы мы выдали 24 "
      "и спрятали, кто написал файл.")

    CH("Kenga пишет C")
    P("c_seed.kenga и expr_c.kenga — зародыш кодогенерации. Язык печатает "
      "C99, его можно скомпилировать снаружи. lower_c идёт дальше: "
      "while, if, for, списки, struct, else if, f64, import.")
    P("rt_kval + lower_kv — tagged runtime: строки, hetero-списки, "
      "события, файловый ввод-вывод. bc_src_c разбирает исходник в "
      "байткод и выплёвывает C-VM. На этой сетке уже сидят kenga_net и "
      "kenga_birth.")
    P("bc_src_c пробрасывает host-argc/argv в VM как `g_kargc`/`g_kargv`. "
      "Любая .kenga может звать argc(), arg(i), file_exists(p), read_line() "
      "нативно (opcodes 106–109). Гостевой argv пропускает путь к more — "
      "host даёт свою папку, гость читает свои пути.")
    P("Это всё ещё C на выходе. Свобода здесь в авторе: исходник "
      "генератора — .kenga, не codegen.rs. Когда VM more покроет тот же "
      "контур без gcc, ступень можно вычеркнуть.")

    PART("Память и тензор")
    CH("Prophet Memory")
    P("Prophet — живая память агента: memory, learn, predict, unroll, "
      "surprise, consolidate. Сохранение ума — save_mind / load_mind, "
      "формат .km совместим с Rust-путём. Реализация без Rust: "
      "kenga/emit/rt_prophet.kenga.")
    P("Учебные файлы: examples/ml/world_model.kenga, surprise_gate.kenga. "
      "Диалог: kenga chat --lite minds/agent.km. Это world-model на "
      "маленьких рядах, не языковая модель из части V.")
    P("ttl и консолидация входят в семантику языка, потому что агент без "
      "забывания — это лог, а не память. В Z-системе коллапс спектра — "
      "физика забвения весов. Здесь ttl — бытовой рычаг той же мысли: "
      "не всё стоит помнить вечно.")

    CH("Тензор и лента")
    P("Плотный f64-тензор: t_from, matmul, поэлементные операции, "
      "reshape, softmax. Источник: kenga/emit/rt_tensor.kenga. Проверка: "
      "examples/ml/tensor_core.kenga.")
    P("Обратный проход — tape, rt_tape.kenga. Примеры: "
      "autograd_tape.kenga, mlp_autograd.kenga. SGD: train_sgd.kenga, "
      "t_sgd_step. Картинка: load_ppm, t_mean. Звук: load_wav. Fusion "
      "кадра и тона — в multimodal-примерах, не в отдельном «CLIP».")
    PRE(
        "let I = t_from([2, 2], [1, 0, 0, 1]);\n"
        "let v = t_from([2, 1], [3, 4]);\n"
        "let y = t_matmul(I, v);\n"
        "# ожидай [3, 4]",
        "единичная матрица")
    P("Vis-bias у word-decoder сидит на логитах (whead @ fuse), не в "
      "магии эмбеддинга. whead учим только на токене цвета, не на каждом "
      "слове подписи. Иначе модель начинает орать цветом в каждом месте.")

    CH("XOR на list/f64")
    P("kenga_net.kenga — маленький MLP на списках, без тензорного хоста. "
      "Это ступень лестницы: алгоритм сети выражен в языке, а не в "
      "вызове matmul из runtime. more этот файл уже исполняет.")
    P("Зачем XOR, если есть decoder. Потому что XOR проверяет, что "
      "градиент и нелинейность живут в диалекте. Decoder проверяет, что "
      "тот же диалект тянет внимание и голову. Путать эти проверки — "
      "как считать телепортацию доказанной, потому что SVD сошёлся.")

    PART("Своя модель")
    CH("Decoder, не скачанный граф")
    P("Половина Grok — не другой алгоритм. Это тот же decoder: внимание, "
      "residual, FFN, норма, LM-head — с другими числами. Kenga уже "
      "выражает эту машину. Числа пока игрушечные.")
    TBL(["Файл", "Что делает"], [
        ["kenga_lm.kenga", "decoder на закрытом словаре"],
        ["kenga_charlm.kenga", "тот же decoder, корпус = .kenga"],
        ["kenga_trigram.kenga", "триграмма на list/i64"],
        ["kenga_birth.kenga", "suffix-LM пишет программу"],
        ["kenga_mm_lm.kenga", "PPM+WAV → подпись (linear)"],
        ["kenga_mm_words.kenga", "цвет = один токен + фраза"],
    ], [4.6 * cm, 7.8 * cm])
    P("Актуальные размеры word-decoder в kenga_dec.kenga: D=16, L=1, "
      "FF=24, CTX=12. Словарь — двенадцать токенов: девять слов подписи "
      "плюс zhivet, v, yazyke. Веса: minds/kenga_mm_we.kt и соседи, в git "
      "не кладём. Образец текста: examples/ml/kenga_mm_words_sample.txt.")
    P("Промпт языковой строки: [\"kenga\", \"zhivet\"], два шага, assert "
      "на yazyke. Строку языка крутим втрое чаще за эпоху, иначе её "
      "топит «kenga vidit».")

    CH("Подписи, которые модель умеет")
    P("Рабочие фразы сейчас такие:")
    PRE(
        "kenga vidit krasnyj kadr i slyshit ton\n"
        "kenga vidit zelenyj kadr i slyshit ton\n"
        "kenga vidit sinij kadr i slyshit ton\n"
        "kenga zhivet v yazyke",
        "корпус подписей")
    P("Визуальные подписи идут через generate_words с vis-bias на первом "
      "токене. Текстовая строка — generate_text без vis. Стебель цвета "
      "в kenga_mm_gen.kenga: kra / ze / si.")
    P("<b>F6.</b> Модель на двенадцати токенах может закончить "
      "«kenga zhivet v yazyke». Это факт про замкнутый словарь, не про "
      "понимание языка.")

    CH("Честный масштаб")
    P("Что доказано: язык описывает и гоняет архитектуру большого LM без "
      "Python и без PyTorch. Обучение next-token есть. Корпус крошечный.")
    TBL(["Нужно для «половины Grok»", "Статус"], [
        ["Decoder / attn / FFN в .kenga", "есть"],
        ["Обучение next-token", "есть, крошечный корпус"],
        ["Словарь 50k–128k, контекст 4k–32k", "нет"],
        ["L≈32–64, D≈4096, MoE", "нет"],
        ["Триллионы токенов", "нет данных"],
        ["GPU-ядра", "нет"],
        ["Чужие веса GGUF как доказательство", "намеренно нет"],
    ], [7.4 * cm, 5.0 * cm])
    P("GTX 1660 SUPER в машине автора часто занята под завязку "
      "(~5800/6144 МиБ). Растить D и L на CPU в этой ситуации — трата "
      "вечера. Когда карта свободна, меняются fn D() и fn L() в "
      "kenga_dec.kenga и корпус. Не раньше.")
    P("<b>О1.</b> Называть текущий decoder половиной Grok — ложь. "
      "<b>О2.</b> Класть игрушечные веса в Kenga-ai/kenga-mm — ложь. "
      "<b>О3.</b> Считать VSIX доказательством языка — путаница слоя.")
    Q("Без GPU и корпуса «половина меня» не появится, какой бы синтаксис "
      "ни был. Появится, когда этот же decoder жрёт реальные веса и железо.")

    PART("Практикум")
    CH("Задачи")
    P("Ответы не подглядывай в examples/ сразу. Проверка: "
      "kenga run --lite. Нумерация совпадает с docs/EXERCISES.md.")
    H2("E1. Арифметика")
    P("Сумма чисел от 1 до 100, println, return 0. Ожидай 5050. Файл: "
      "examples/exercises/e01_sum.kenga.")
    H2("E2. Список")
    P("[3, 1, 4, 1, 5] — максимум циклом. Шпаргалка после попытки: "
      "lists_lite.kenga.")
    H2("E3. Struct")
    P("Vec2 { x, y }, len2 = x²+y², точка (3,4) → 25. Бонус: прогони "
      "через lower_c.")
    H2("E4. События")
    P("Мини-агент on \"tick\", emit, pump. Цепочка как в agent.kenga.")
    H2("E5. Тензор")
    P("Единичная 2×2 на вектор [3,4]. Ожидай [3,4].")
    H2("E6. SGD")
    P("С нулевого W выучи [1,0] → [2] одним рядом t_sgd_step. "
      "Шпаргалка: train_sgd.kenga.")
    H2("E7. Картинка")
    P("load_ppm(examples/ml/assets/dot.ppm) → t_mean. Три числа около 0.5.")
    H2("E8. Fusion")
    P("Сложи image embedding с t_from([3],[0.1,0.1,0.1]), получи скаляр "
      "через t_matmul с [1,1,1].")
    H2("E9. Birth")
    P("scripts\\kenga-birth.cmd. Модель пишет kenga_born.kenga, запуск "
      "печатает 24. Прочитай kenga_birth.kenga — это suffix-LM.")
    H2("E10–E11. Видишь и слышишь")
    P("kenga_mm_lm.kenga — три подписи про цвет и тон. "
      "kenga_mm_gen.kenga — стебель kra/ze/si. kenga_mm_words.kenga — "
      "цвет словом и строка kenga zhivet v yazyke.")

    CH("Лабораторный журнал")
    P("Короткие записи, не легенда. Они объясняют, почему книга говорит "
      "именно это, а не «язык стремится к независимости».")
    TBL(["Запись", "Что случилось"], [
        ["host = includes",
         "kenga_lite.c сжат до CRT + rt_*.inc.c"],
        ["52 selftest",
         "build.cmd не зелёный — хост не готов"],
        ["XOR на more",
         "kenga_net.kenga исполняется Kenga-VM"],
        ["экраны строк",
         "без \\\" c_seed висел на сотнях МБ"],
        ["birth на more",
         "писал born и сразу печатал 24"],
        ["12 токенов",
         "zhivet не тонет, если крутить ×3"],
        ["vis-bias",
         "учить whead только на токене цвета"],
        ["1660 занята",
         "D/L не растим на CPU впустую"],
    ], [3.6 * cm, 8.8 * cm])
    P("Реестр фактов этой книги: F1 runtime без Rust; F2 маркетплейс "
      "не догоняет язык; F3 C/Rust — подмости; F4 more → 24; F5 birth "
      "не GGUF; F6 двенадцать токенов умеют одну фразу. Три опровержения: "
      "O1 половина Grok; O2 заливка игрушки как большой модели; O3 VSIX "
      "как доказательство.")

    CH("Куда смотреть в репозитории")
    P("Учить за вечер — docs/LEARN.md. Справочник — docs/LANGUAGE.md. "
      "Своя LM — docs/KENGA_LM.md. Hugging Face — docs/HUGGINGFACE.md, "
      "и не заливай туда игрушку. Расширение — docs/MARKETPLACE.md, и не "
      "поднимай версию без правки редактора.")
    P("Сборка книги, которую ты читаешь:")
    PRE(
        "python book\\make_cover.py\n"
        "python book\\make_book.py",
        "пересборка PDF и EPUB")
    P("Исходник текста — book/make_book.py. Если факт в книге расходится "
      "с репозиторием, правь книгу или код, не замазывай абзацем про "
      "«эволюцию ландшафта».")

    add("backmatter")
    P("Первое издание закрывает язык на ступени 3.13: lite-хост из "
      "Kenga, компилятор more, петля birth, игрушечный decoder. Следующее "
      "издание имеет смысл, когда изменятся числа decoder'а или исчезнет "
      "C-хост — не когда накопится ещё одна страница документации.")
    if os.path.exists(BACK):
        add("backcover")


def flow_pdf():
    E = []
    part_n = [1]
    ch_n = [0]

    def paragraph(t, s=st_body):
        E.append(Paragraph(t, s))

    for b in BLOCKS:
        k = b["kind"]
        if k == "front":
            if os.path.exists(COVER):
                E.append(Image(COVER, width=W, height=H))
                E.append(NextPageTemplate("Body"))
                E.append(PageBreak())
            E.append(Spacer(1, 3 * cm))
            paragraph("КЕНГА", ParagraphStyle(
                "t1", fontName="DJVB", fontSize=24, leading=29,
                alignment=TA_CENTER, textColor="#1a3a6b"))
            E.append(Spacer(1, 0.3 * cm))
            paragraph("Язык, который компилирует и учит себя",
                      ParagraphStyle("t2", fontName="DJV", fontSize=11.5,
                                     leading=14, alignment=TA_CENTER,
                                     textColor="#333333"))
            E.append(Spacer(1, 1 * cm))
            paragraph("От hello до модели, которая пишет программу:",
                      ParagraphStyle("t3", fontName="DJVI", fontSize=9.5,
                                     leading=12, alignment=TA_CENTER,
                                     textColor="#555555"))
            paragraph("диалект, хост без Rust, компилятор на Kenga<br/>"
                      "и честный масштаб своей LM",
                      ParagraphStyle("t4", fontName="DJVI", fontSize=9.5,
                                     leading=12, alignment=TA_CENTER,
                                     textColor="#555555"))
            E.append(Spacer(1, 2.8 * cm))
            paragraph("Герман Янтарас", ParagraphStyle(
                "t5", fontName="DJVB", fontSize=13, leading=16,
                alignment=TA_CENTER))
            paragraph("при участии kenga-lite 3.13", ParagraphStyle(
                "t6", fontName="DJV", fontSize=8.5, leading=11,
                alignment=TA_CENTER, textColor="#888888"))
            E.append(Spacer(1, 1.8 * cm))
            paragraph("Первое издание · 2026", ParagraphStyle(
                "t7", fontName="DJV", fontSize=9, leading=12,
                alignment=TA_CENTER, textColor="#888888"))
            E.append(PageBreak())
        elif k == "pagebreak":
            E.append(PageBreak())
        elif k == "toc":
            E.append(Paragraph("Оглавление", st_h1))
            toc = TableOfContents()
            toc.levelStyles = [_TOC_ST(0), _TOC_ST(1)]
            E.append(toc)
            E.append(PageBreak())
        elif k == "intro":
            p = Paragraph(b["text"], ParagraphStyle("ct0", parent=st_h1,
                                                    fontSize=14))
            p._toc = b["text"]
            E.append(p)
        elif k == "part":
            E.append(PageBreak())
            E.append(Spacer(1, 4 * cm))
            E.append(Paragraph(
                f"ЧАСТЬ {roman(part_n[0])}",
                ParagraphStyle("pn", parent=st_part, fontSize=13,
                               textColor="#888888")))
            E.append(Spacer(1, 0.4 * cm))
            p = Paragraph(b["text"], st_part)
            p._toc = f"<b>Часть {roman(part_n[0])}. {b['text']}</b>"
            E.append(p)
            part_n[0] += 1
        elif k == "ch":
            ch_n[0] += 1
            E.append(PageBreak())
            E.append(Paragraph(
                f"Глава {ch_n[0]}",
                ParagraphStyle("cn", fontName="DJV", fontSize=9,
                               textColor="#888888", spaceAfter=2)))
            p = Paragraph(b["text"], ParagraphStyle("ct", parent=st_h1,
                                                    fontSize=14))
            p._toc = f"{ch_n[0]}. {b['text']}"
            E.append(p)
        elif k == "h2":
            E.append(Paragraph(b["text"], st_h2))
        elif k == "p":
            paragraph(b["text"])
        elif k == "q":
            paragraph("«" + b["text"] + "»", st_quote)
        elif k == "tbl":
            data = [[Paragraph(h, st_tblh) for h in b["headers"]]]
            for r in b["rows"]:
                data.append([Paragraph(str(c), st_tbl) for c in r])
            t = Table(data, colWidths=b.get("widths") or [None] * len(b["headers"]),
                      repeatRows=1)
            t.setStyle(TableStyle([
                ("BACKGROUND", (0, 0), (-1, 0), "#dbe5f1"),
                ("ROWBACKGROUNDS", (0, 1), (-1, -1), ["#ffffff", "#f4f7fb"]),
                ("GRID", (0, 0), (-1, -1), 0.4, "#b8cce4"),
                ("VALIGN", (0, 0), (-1, -1), "TOP"),
                ("LEFTPADDING", (0, 0), (-1, -1), 3),
                ("RIGHTPADDING", (0, 0), (-1, -1), 3),
            ]))
            E.append(t)
            E.append(Spacer(1, 6))
        elif k == "pre":
            if b.get("title"):
                E.append(Paragraph(b["title"], st_cap))
            E.append(Preformatted(b["text"], st_mono))
            E.append(Spacer(1, 6))
        elif k == "backmatter":
            E.append(PageBreak())
            p = Paragraph("Послесловие", ParagraphStyle(
                "ctend", parent=st_h1, fontSize=14))
            p._toc = "Послесловие"
            E.append(p)
        elif k == "backcover":
            E.append(NextPageTemplate("Cover"))
            E.append(PageBreak())
            E.append(Image(BACK, width=W, height=H))
    return E


def rl_to_html(t):
    t = t.replace("<br/>", "\n")
    t = re.sub(r"<b>(.*?)</b>", r"<strong>\1</strong>", t)
    t = re.sub(r"<i>(.*?)</i>", r"<em>\1</em>", t)
    t = t.replace("\n", "<br/>")
    return t


def write_epub():
    book = epub.EpubBook()
    book.set_identifier("urn:kenga:yantaras:v1")
    book.set_title("Кенга. Язык, который компилирует и учит себя")
    book.set_language("ru")
    book.add_author("Герман Янтарас")
    book.add_metadata("DC", "date", "2026")
    book.add_metadata("DC", "rights", "© Герман Янтарас, 2026")

    css = epub.EpubItem(
        uid="style", file_name="style/book.css", media_type="text/css",
        content="""
body { font-family: Georgia, "Times New Roman", serif; line-height: 1.45;
       color: #222; margin: 1.2em; }
h1 { color: #1a3a6b; font-size: 1.4em; }
h2 { color: #1a3a6b; font-size: 1.15em; margin-top: 1.4em; }
h3 { color: #2a5aa8; font-size: 1.02em; }
p { text-align: justify; }
blockquote { color: #444; font-style: italic; margin: 0.8em 1.2em; }
pre { font-family: Consolas, "DejaVu Sans Mono", monospace; font-size: 0.82em;
      background: #f4f7fb; padding: 0.7em; white-space: pre-wrap; }
table { border-collapse: collapse; width: 100%; font-size: 0.88em; margin: 0.8em 0; }
th { background: #dbe5f1; text-align: left; padding: 0.35em 0.45em; }
td { border: 1px solid #b8cce4; padding: 0.35em 0.45em; vertical-align: top; }
tr:nth-child(even) td { background: #f4f7fb; }
.part { text-align: center; color: #1a3a6b; margin-top: 3em; }
.muted { color: #888; text-align: center; }
""".encode("utf-8"))
    book.add_item(css)

    if os.path.exists(COVER):
        with open(COVER, "rb") as f:
            book.set_cover("cover.png", f.read())

    chapters = []
    buf = []
    title = "Титул"
    file_i = [0]

    def flush():
        if not buf:
            return
        file_i[0] += 1
        name = f"c{file_i[0]:02d}.xhtml"
        ch = epub.EpubHtml(title=title, file_name=name, lang="ru")
        ch.content = "\n".join(buf)
        ch.add_item(css)
        book.add_item(ch)
        chapters.append(ch)
        buf.clear()

    part_n = 1
    ch_n = 0
    for b in BLOCKS:
        k = b["kind"]
        if k == "front":
            buf.append("<p class='muted'>КЕНГА</p>")
            buf.append("<h1>Язык, который компилирует и учит себя</h1>")
            buf.append("<p class='muted'>Герман Янтарас<br/>"
                       "при участии kenga-lite 3.13<br/>"
                       "Первое издание · 2026</p>")
        elif k == "intro":
            flush()
            title = b["text"]
            buf.append(f"<h1>{html.escape(b['text'])}</h1>")
        elif k == "part":
            flush()
            title = f"Часть {roman(part_n)}. {b['text']}"
            buf.append(f"<p class='muted'>ЧАСТЬ {roman(part_n)}</p>")
            buf.append(f"<h1 class='part'>{html.escape(b['text'])}</h1>")
            part_n += 1
        elif k == "ch":
            flush()
            ch_n += 1
            title = f"{ch_n}. {b['text']}"
            buf.append(f"<p class='muted'>Глава {ch_n}</p>")
            buf.append(f"<h2>{html.escape(b['text'])}</h2>")
        elif k == "h2":
            buf.append(f"<h3>{html.escape(b['text'])}</h3>")
        elif k == "p":
            buf.append(f"<p>{rl_to_html(b['text'])}</p>")
        elif k == "q":
            buf.append(f"<blockquote><p>{html.escape(b['text'])}</p></blockquote>")
        elif k == "tbl":
            rows = "<tr>" + "".join(
                f"<th>{html.escape(h)}</th>" for h in b["headers"]) + "</tr>"
            for r in b["rows"]:
                rows += "<tr>" + "".join(
                    f"<td>{html.escape(str(c))}</td>" for c in r) + "</tr>"
            buf.append(f"<table>{rows}</table>")
        elif k == "pre":
            src = html.escape(b["text"])
            cap = html.escape(b["title"]) if b.get("title") else ""
            if cap:
                buf.append(f"<p><em>{cap}</em></p>")
            buf.append(f"<pre>{src}</pre>")
        elif k == "backmatter":
            flush()
            title = "Послесловие"
            buf.append("<h1>Послесловие</h1>")
        elif k in ("pagebreak", "toc", "backcover"):
            continue
    flush()

    book.toc = [epub.Link(ch.file_name, ch.title, ch.file_name) for ch in chapters]
    book.add_item(epub.EpubNcx())
    book.add_item(epub.EpubNav())
    book.spine = ["nav"] + chapters
    epub.write_epub(EPUB_OUT, book)


def main():
    if not os.path.exists(COVER):
        if HERE not in sys.path:
            sys.path.insert(0, HERE)
        from make_cover import _back, _front
        _front()
        _back()
    write_content()
    doc = Book(
        PDF_OUT, pagesize=A5,
        leftMargin=M, rightMargin=M, topMargin=1.7 * cm, bottomMargin=1.5 * cm,
        title="Кенга: язык, который компилирует и учит себя",
        author="Герман Янтарас",
    )
    frame_cover = Frame(0, 0, W, H, id="cover", leftPadding=0, rightPadding=0,
                        topPadding=0, bottomPadding=0)
    frame_body = Frame(M, 1.5 * cm, W - 2 * M, H - 3.2 * cm, id="body")
    doc.addPageTemplates([
        PageTemplate(id="Cover", frames=[frame_cover]),
        PageTemplate(id="Body", frames=[frame_body], onPage=on_page),
    ])
    doc.multiBuild(flow_pdf())
    write_epub()
    print("PDF", PDF_OUT)
    print("EPUB", EPUB_OUT)


if __name__ == "__main__":
    os.chdir(HERE)
    main()
