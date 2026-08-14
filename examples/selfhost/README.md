# Self-host ladder (pure Kenga)

| Step | File | What |
|---|---|---|
| 1 | `arith.kenga` | expr eval |
| 2 | `mini.kenga` | variables |
| 3 | `iffy.kenga` | if / cmp |
| 4 | `loopfn.kenga` | while + fn |
| 5 | `bytecode.kenga` | stack VM + emit assign/expr |
| 6 | `bc_while.kenga` | bytecode while via JMP/JMPF |
| 7 | `bc_fn.kenga` | **bytecode functions** CALL/RET |

```bash
kenga run examples/selfhost/bc_fn.kenga
```

Next: compile a real subset of `.kenga` syntax onto this VM → chicken-egg.
