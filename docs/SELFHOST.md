# Self-host seed

Полный компилятор Kenga на Kenga — долгий путь. Сейчас в репо есть **зерно**:

`examples/selfhost/arith.kenga` — лексер + recursive descent + eval арифметики
(`2+3*4`, скобки, унарный минус) целиком на чистом Kenga.

```bash
kenga run examples/selfhost/arith.kenga
```

Дальше по лестнице:
1. выражения / statements (let, if) как AST-списки
2. bytecode emitter на Kenga
3. VM на Kenga (или эмиссия в уже существующий C/LLVM)
4. компиляция собственного исходника → chicken-egg закрыт

Rust-bootstrap остаётся хостом, пока п.4 не зелёный.
