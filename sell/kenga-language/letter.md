# Сопроводительное письмо — Kenga Language Stack

## Русский

**Тема:** Kenga: язык, компилятор на себе, VM и C99. 30 минут с вашим compiler lead

Я пишу вам, потому что вы держите компилятор, рантайм или developer platform. Не потому что закупаете «AI». Это языковой стек.

Kenga — язык, компилятор, написанный на Kenga, байткод-VM и выпуск в C99. Lite-диалект собирается в один exe, kenga-lite. Без pip. Без cargo. Компилятор называется more.kenga. lower_c и bc_src_c пишут C. Лестница self-host из 13 шагов существует. Бутстрап всё ещё C. Лицензия MIT, репозиторий github.com/GermannM3/kenga-lang.

Сразу про дыры, чтобы вы не искали их сами. Полный self-host без C-хоста не сделан. Каталог src/ на Rust — наследие; новый код туда не идёт. Это не замена LLVM. Это не CUDA. 25 августа 2026 у нас прошёл freedom-smoke. Спека — docs/SPEC.md, короткий тур — docs/TOUR.md. Как слезаем с Rust — docs/INDEPENDENCE.md. Текст, который мы даём своим, — docs/FOR_FRIENDS.md.

Три pptx (русский, английский, китайский) лежат в этой папке, рядом с письмом.

Предложение без дыма. Лицензировать стек или купить его. Либо четыре недели встройки: вы приносите языковое подмножество, мы на вашей машине показываем compile и run. Либо инженерное партнёрство, чтобы вытеснить ещё кусок C-хоста.

Мне нужны 30 минут с человеком, который у вас отвечает за компилятор или рантайм. Если склонируете репо, прогоните smoke и станет скучно — напишите это. Лучше честный отказ от правильной команды, чем вежливое «интересно» от чужой.

## English

**Subject:** Kenga: a language, its compiler, a VM, and C99. 30 minutes with your compiler lead

I am writing because you run a compiler, a runtime, or a developer platform. Not because you buy AI. This is a language stack.

Kenga is the language, a compiler written in Kenga, a bytecode VM, and emit down to C99. The lite dialect is one executable, kenga-lite. No pip. No cargo. The compiler is more.kenga. lower_c and bc_src_c write C. There are 13 self-host steps. C is still the boot stage. MIT. github.com/GermannM3/kenga-lang.

I'll name the holes first, so you do not trip on them. Full self-host without a C host is not done. The Rust tree under src/ is legacy; we do not put new work there. This is not an LLVM replacement and it is not CUDA. On 2026-08-25 freedom-smoke passed. Spec and tour are in docs/SPEC.md and docs/TOUR.md. How we get off Rust is docs/INDEPENDENCE.md. For people we know: docs/FOR_FRIENDS.md.

The three pptx files (Russian, English, Chinese) are in this folder, next to this letter.

License the stack or acquire it. Or four weeks of embed: you bring a language subset, we show compile and run on your machine. Or an engineering partnership to eat more of the remaining C host.

I want 30 minutes with the person who owns your compiler or runtime. If you clone the repo, run the smoke, and it is a waste of time, tell me. I'd rather hear it from the right team than a polite maybe from the wrong one.

## 中文

**主题：** Kenga 语言栈：语言、自举编译器、字节码 VM、落到 C99。想和你们编译器负责人聊 30 分钟

写信是因为你们管编译器、运行时，或者开发者平台。不是来推「AI 产品」的。这是语言栈。

Kenga 是语言本身，编译器用 Kenga 写（more.kenga），带字节码虚拟机，emit 到 C99。Lite 方言打成一个 exe，叫 kenga-lite，不依赖 pip，也不依赖 cargo。lower_c 和 bc_src_c 负责吐 C。Self-host 梯子有 13 级。启动层现在还是 C。许可证 MIT，仓库 github.com/GermannM3/kenga-lang。

不好听的先说。没有 C host 的完整 self-host，没做完。src/ 里的 Rust 是遗留代码，新工作不往里加。这不是来换你们 LLVM 的，更不是 CUDA。2026-08-25 这天 freedom-smoke 是过的。规格和导览在 docs/SPEC.md、docs/TOUR.md。怎么离开 Rust 写在 docs/INDEPENDENCE.md。给熟人看的直白话在 docs/FOR_FRIENDS.md。

三个 pptx（俄 / 英 / 中）就在这个目录里，和这封信放一起。

合作就这几条。授权，或者把栈买下来。或者四周：你们出一份语言子集，我们在你们机器上把编译和运行跑通。或者一起做工程，把剩下的 C host 再削一层。

想和真正管编译器或运行时的人谈 30 分钟。clone 完跑 smoke，如果觉得没意思，直接回一句。比找错组、客套拒绝要有用。
