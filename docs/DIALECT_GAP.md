# Dialect gap (2026-08-30)

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
| match | exp. | **yes** (i64 stmt) | **yes** (i64 + str) | **yes** (i64 stmt) | **yes** (i64 stmt) | yes (VM no) |
| enum | exp. | **yes** (unit) | **yes** (unit) | **yes** (unit) | **yes** (unit) | yes (ident) |
| compound `+=` | no | **yes** | **yes** | **yes** | **yes** | yes |
| stepped `0..10 step 2` | no | **yes** | **yes** | **yes** | **yes** | yes |
| `true`/`false` | yes | yes | yes | yes | yes | yes |
| `%` | yes | yes | yes | yes | yes | yes |
| `&&` `\|\|` | yes | **JMPF skip** | **JMPF skip** | **JMPF skip** | C `&&` | yes |
| `slice` / `index_of` / `starts_with` / `split` | no | **yes** | **yes** | **yes** (str) | no | no |
| map / json_set | no | no | **yes** | no | no | no |
| match string arm | exp. | no | **yes** | no | no | yes (parse) |
| getenv / http / json | no | **yes** | **yes** | **yes** | no | no |

Closed this pass: bitwise + hex/bin; `+=` / `step`; short-circuit; integer `match` and unit `enum`; string `slice`/`index_of`/`starts_with`/`split`; host I/O; `map_*` / `json_set`; string `match` on more; `lite.kenga` `for i in a..b` + `break`/`continue` + `match` i64.
