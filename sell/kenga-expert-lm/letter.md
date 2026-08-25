# Kenga Expert LM: letter

Outbound. Same facts in all three languages. Bind x17.6 and Axis C 0/6 exact live in the same letter. M5.3 card vs M6 morning 2026-08-25 are labeled separately. Do not claim trajectory-not-data.

---

## English

**Subject:** Bind compile x17.6 on 838k params. Real-file exact stdout is still 0/6.

**Alt subject:** Tiny expert method, compiler in the loop. Not a chat product.

Kenga Expert LM is a method for tiny expert models. Not a chat product. You get a language, a compiler that runs the program, a factory of already-executed examples, and a decoder that fits on a laptop CPU. Published weights show that loop, not a general assistant.

Card M5.3 (huggingface.co/GermannM, kenga-prophet-m5-3): about 838k parameters, K=128 D=128 H=8 L=6. Batch 64, 2400 steps, about 3h on CPU. Same backbone as M5.2. Only the training distribution changed.

Axis A, factory compile: 92.5% -> 100%.
Axis B, bind compile: 5.6% -> 98.6% (x17.6).
Axis C, real-file compile: 0/6 -> 1/6. Exact stdout: 0/6.

Keep both numbers if you quote us. Binding among same-signature distractors moved. Held-out human files still miss the original stdout.

Repair rp0, after an eval-script bugfix: fixed@1 6.7% / pass@4 21.7%. A task-boundary marker is a control, not a spell.

M6 morning 2026-08-25 is a different run. Label the date. Held-out overall 72.57%. NL-to-code: compile 50%, greedy match 1.2%. Genesis is CLOSED: realgen T1 compile 0%.

We do not claim the gain is trajectory rather than data. Seed-control is not done.

For on-device assistants, internal-language helpers, and labs that score with a compiler in the loop.

Three offers: acquire the stack; a four-week pilot on your language, expert up to about 2M parameters, success criterion written before we start; research partnership on verifier-gated growth, with Genesis blocked until realgen moves.

Code: github.com/GermannM3/kenga-lang

---

## Russian

**Subject:** Bind compile вырос в 17.6 раза. По живым файлам точный stdout: 0/6.

**Alt subject:** Метод крошечного эксперта, компилятор в цикле оценки. Это не чат.

Kenga Expert LM: метод для маленьких экспертных моделей. Не чат. Вы получаете язык, компилятор, который программу реально запускает, фабрику примеров, уже прогнанных рантаймом, и декодер, который живёт на ноутбучном процессоре. Опубликованные веса показывают этот контур. Универсальным ассистентом они не являются.

Карточка M5.3 (huggingface.co/GermannM, kenga-prophet-m5-3): около 838 тысяч параметров, K=128 D=128 H=8 L=6. Batch 64, 2400 шагов, примерно три часа на CPU. То же тело, что у M5.2. Поменяли только распределение данных.

Ось A, factory compile: 92.5% -> 100%.
Ось B, bind compile: 5.6% -> 98.6% (x17.6).
Ось C, compile по живым файлам: 0/6 -> 1/6. Точный stdout: 0/6.

Если цитируете, берите оба числа. Привязка среди чужих функций с той же сигнатурой сдвинулась. Свободное продолжение чужих файлов исходный вывод по-прежнему не воспроизводит.

Ремонт rp0, после бага в скрипте оценки: fixed@1 6.7% / pass@4 21.7%. Маркер границы задачи: это контрольный опыт, не фокус.

Утренний отчёт M6 от 2026-08-25: другой прогон. Дату подписывайте. Held-out overall 72.57%. NL-to-code: compile 50%, greedy match 1.2%. Genesis CLOSED: realgen T1 compile 0%.

Не утверждаем тезис «траектория, а не данные». Seed-control не ставили.

Письмо для тех, кому нужен эксперт на устройстве, помощник по внутреннему языку, лаборатория, которая считает генерацию компилятором в цикле.

Три входа: покупка стека; пилот на четыре недели на вашем языке, эксперт до ~2M параметров, критерий успеха до старта; исследовательское партнёрство по росту под верификатором. Genesis сейчас закрыт, пока не сдвинется realgen.

Код: github.com/GermannM3/kenga-lang

---

## Chinese

**Subject:** 绑定编译 17.6 倍；真实文件精确 stdout 仍是 0/6

**Alt subject:** 小专家模型的方法，编译器在评分环里。不是聊天产品。

Kenga Expert LM 是做小专家模型的方法，不是聊天产品。你拿到的是一门语言、一个会真跑程序的编译器、一座已经用运行时验过的语料工厂，以及能在笔记本 CPU 上活下来的解码器。公开权重是这条闭环的演示，不是通用助手。

M5.3 卡片（huggingface.co/GermannM，kenga-prophet-m5-3）：约 83.8 万参数，K=128 D=128 H=8 L=6。batch 64，2400 步，CPU 大约三小时。和 M5.2 同一具骨干，只改了训练数据的分布。

轴 A，工厂编译：92.5% -> 100%。
轴 B，绑定编译：5.6% -> 98.6%（x17.6）。
轴 C，真实文件编译：0/6 -> 1/6。精确 stdout：0/6。

要引用就把两个数一起引。同签名干扰里选对函数，这件事动了。拿着别人的文件自由续写，仍然对不上原来的输出。

修复 rp0，评估脚本的 bug 修好之后：fixed@1 6.7% / pass@4 21.7%。任务边界标记是对照实验，不是咒语。

M6 早晨报告，日期 2026-08-25，是另一次运行。引用时写上日期。held-out 总体 72.57%。NL-to-code：compile 50%，greedy match 1.2%。Genesis 关闭：realgen T1 compile 0%。

我们不声称这是「轨迹而不是数据」。对照种子的实验还没做。

写给端侧助手、内部语言、以及用编译器在环里打分的实验室。

三条路：买下整栈；四周试点，做你们的语言，专家模型大约到 2M 参数，通过标准开工前写好；研究合作走验证器把关的增长。Genesis 目前关掉，realgen 不动就不谈开门。

代码：github.com/GermannM3/kenga-lang
