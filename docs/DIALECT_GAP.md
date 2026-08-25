# Dialect gap (2026-08-25)

Lab Dialect. Columns: **SPEC** = portable subset (`docs/SPEC.md`); **lite** = `kenga/emit/rt_*.kenga` ? `kenga-lite`; **more** = `kenga/compiler/more.kenga`; **bc** = `kenga/emit/bc_src_c.kenga`; **lower_c** = `kenga/emit/lower_c.kenga`; **rust** = `src/parser.rs` + lexer (read-only).

| Feature | SPEC | lite | more | bc | lower_c | rust |
|---|---|---|---|---|---|---|
| bitwise `& ^ \| ~ << >>` | full VM | **yes** | **yes** | **yes** | **yes** | yes |
| hex/bin `0x` `0b` | full only | **yes** (`rt_parse`) | **yes** | **yes** | **yes** | yes |
| for-in-list | yes | yes | yes | yes | yes | yes |
| for-range `a..b` | yes | yes | yes | yes | yes | yes |
| break/continue | yes | yes | yes | yes | yes | yes |
| elif `else if` | yes | yes | yes | yes | yes | yes |
| struct | exp. | yes | yes | yes | yes | yes |
| import | exp. | yes (flatten) | yes | yes | yes | yes |
| events `on`/`emit`/`pump` | exp. | yes | yes | yes | yes | yes |
| argc/arg | no | yes | yes | yes | **yes** | no (call) |
| match | exp. | no | no | no | no | yes (VM no) |
| enum | exp. | no | no | no | no | yes (ident) |
| compound `+=` | no | no | no | no | no | yes |
| stepped `0..10 step 2` | no | no | no (step=1) | no (step=1) | no (step=1) | yes |
| `true`/`false` | yes | yes | yes | yes | yes | yes |
| `%` | yes | yes | yes | yes | yes | yes |
| `&&` `\|\|` | yes (eager on lite/more) | yes | yes | yes | yes | yes |

Closed this pass: one dialect for bitwise + hex/bin on lite, more, lower_c, bc_src_c. Next: `+=` desugar, stepped range -- not match/enum.
