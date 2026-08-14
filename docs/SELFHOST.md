# Self-host

На чистом Kenga уже есть **Kenga-lite → bytecode VM**:

`let` / `while` / `if-else` / `fn` / `return` / вызовы — см. `examples/selfhost/kenga_lite.kenga`.

До полного chicken-egg осталось: подключить этот пайплайн как `kenga run` для подмножества, расширить синтаксис (struct, import, Memory).
