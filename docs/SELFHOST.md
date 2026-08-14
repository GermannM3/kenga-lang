# Self-host seed

Полный компилятор Kenga на Kenga — долгий путь. Сейчас в репо есть **зерно**:

`examples/selfhost/arith.kenga` — арифметика  
`examples/selfhost/mini.kenga` — переменные (`x=2+3; y=x*4; y`)  
`examples/selfhost/iffy.kenga` — **if / сравнения** (`if x>2 then 10 else 20`)

```bash
kenga run examples/selfhost/arith.kenga
kenga run examples/selfhost/mini.kenga
kenga run examples/selfhost/iffy.kenga
```

Дальше по лестнице:
1. ~~выражения / assign / if~~  
2. `while` + функции в мини-языке  
3. bytecode emitter на Kenga  
4. chicken-egg закрыт

Rust-bootstrap остаётся хостом, пока п.4 не зелёный.
