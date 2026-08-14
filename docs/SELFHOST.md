# Self-host

Лестница в `examples/selfhost/` — всё на чистом Kenga:

1. ~~arith~~ → ~~mini vars~~ → ~~if~~ → ~~while/fn~~  
2. ~~stack bytecode VM + emit~~ → ~~bytecode while~~  
3. **next:** bytecode functions, затем компиляция подмножества настоящего `.kenga`

Rust-bootstrap остаётся хостом до закрытия chicken-egg.
