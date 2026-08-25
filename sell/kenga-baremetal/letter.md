# Сопроводительное письмо — Kenga Freestanding

## Русский
**Тема:** C без libc: emit, который стыдно прятать malloc

Freestanding — для тех, кто уже линкует -nostdlib. Kenga пишет логику, emit-c --freestanding даёт C99 без stdio/stdlib. Аллокация через слабый kf_alloc: нет хука — первый alloc умирает. Так проще аудиту, чем «рантайм сам как-нибудь».

Это не ОС и не RTOS-сертификат. Хуки ядра ваши. CUDA нет. На встрече читаем generated .c, колоду можно не открывать.

Три pptx в папке. Канон — docs/FREESTANDING.md.

Просьба: 30 минут с kernel/firmware. Пусть принесут свой аллокатор — к нему и будем стыковать.

Герман
https://github.com/GermannM3/kenga-lang

## English
**Subject:** C without libc — emit that will not hide malloc

Freestanding is for people who already link -nostdlib. Kenga writes the logic; emit-c --freestanding yields C99 without stdio/stdlib. Alloc goes through weak kf_alloc: no hook, first alloc dies. Easier to audit than a runtime that shrugs.

Not an OS. Not an RTOS certificate. You write kernel hooks. No CUDA. On the call we read generated .c. The deck is optional.

Three pptx files. Canon is docs/FREESTANDING.md.

Ask: 30 minutes with kernel/firmware. Bring your allocator. That is the join.

Hermann
https://github.com/GermannM3/kenga-lang

## 中文
**主题：** 没有 libc 的 C：不愿藏起 malloc 的 emit

给已经在用 -nostdlib 的人。Kenga 写逻辑，emit-c --freestanding 出无 stdio/stdlib 的 C99。分配走弱符号 kf_alloc：没有钩子，第一次分配就死。比「运行时自己看着办」好审计。

不是操作系统，不是 RTOS 认证。内核钩子你们写。没有 CUDA。会上读生成的 .c，片子可以不打开。

三份 pptx。正文是 docs/FREESTANDING.md。

请求：和内核/固件谈 30 分钟。带上你们的分配器，就接这个。

Hermann
https://github.com/GermannM3/kenga-lang
