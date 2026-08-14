# Self-host ladder (pure Kenga)

| Step | File | What |
|---|---|---|
| 1 | `arith.kenga` | expr eval |
| 2 | `mini.kenga` | variables |
| 3 | `iffy.kenga` | if / cmp |
| 4 | `loopfn.kenga` | while + fn |
| 5 | `bytecode.kenga` | stack VM + emit assign/expr |
| 6 | `bc_while.kenga` | bytecode while via JMP/JMPF |
| 7 | `bc_fn.kenga` | bytecode functions CALL/RET |
| 8 | `kenga_lite.kenga` | **диалект ближе к `.kenga`**: `let` / `while` / `if` / `fn` / `return` |

```bash
kenga run examples/selfhost/kenga_lite.kenga
```

На VM уже крутится:

```
fn main() -> i64 {
  let n = 1;
  let i = 1;
  while i <= 5 { n = n * i; i = i + 1; }
  return n;
}
```
