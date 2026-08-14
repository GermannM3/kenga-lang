# Self-host ladder (pure Kenga)

| Step | File | What |
|---|---|---|
| 1 | `arith.kenga` | expr eval |
| 2 | `mini.kenga` | variables |
| 3 | `iffy.kenga` | if / cmp |
| 4 | `loopfn.kenga` | while + fn |
| 5 | `bytecode.kenga` | stack VM + emit assign/expr |
| 6 | `bc_while.kenga` | bytecode while via JMP/JMPF |

```bash
kenga run examples/selfhost/loopfn.kenga
kenga run examples/selfhost/bytecode.kenga
kenga run examples/selfhost/bc_while.kenga
```

Next: functions on bytecode, then emit real `.kenga` subset → chicken-egg.
