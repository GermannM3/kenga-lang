# Self-host

## Свобода от Rust

```bat
bootstrap\build.cmd
kenga run --lite examples\selfhost\struct_lite.kenga
kenga run --lite examples\selfhost\float_lite.kenga
kenga run --lite examples\hello.kenga
```

```bash
bash bootstrap/build.sh
kenga run --lite examples/hello.kenga
```

Linux/macOS/Git Bash: `docs/UNIX.md`.

`*_lite.kenga` и `examples/selfhost/` автоматически идут в C-lite, если бинарник собран.

Prophet Memory на lite (без Rust):

```bat
bootstrap\bin\kenga-lite.exe run examples\prophet.kenga
kenga run --lite examples\prophet.kenga
```

### Chicken-egg: lite из `.kenga` без ручного C

Tagged emit-c (`KVal`) умеет nested/hetero lists + strings:

```bat
bootstrap\rebuild-from-kenga.cmd
```

Цепочка: `examples/selfhost/kenga_lite.kenga` → `emit-c` → `kenga-lite-gen.exe` (Rust только на машине сборки).

## Лестница на чистой Kenga (lab, хост всё ещё Rust)

| Step | File | What |
|---|---|---|
| 1 | `arith.kenga` | expr eval |
| 2 | `mini.kenga` | variables |
| 3 | `iffy.kenga` | if / cmp |
| 4 | `loopfn.kenga` | while + fn |
| 5 | `bytecode.kenga` | stack VM + emit assign/expr |
| 6 | `bc_while.kenga` | bytecode while via JMP/JMPF |
| 7 | `bc_fn.kenga` | bytecode functions CALL/RET |
| 8 | `kenga/compiler/lite.kenga` | i64 VM: `for i in a..b`, `break`/`continue`, `match` i64 |
| 9 | `kenga_more.kenga` | f64 + lists + println/assert/round |
| 10 | `kenga/compiler/more.kenga` | for / elif / struct / import / events / map / json_set / match str / host I/O |
| 11 | `kenga/emit/lower_c.kenga` | recursive Kenga→C99 (agent, lists, struct, f64, import) |
| 12 | `kenga/emit/lower_kv.kenga` + `rt_kval` | tagged KVal path (str/ord/lex/parse) |
| 13 | `opcodes_c` + `bc_vm_c` + `bc_compile_c` + **`bc_src_c`** | opcodes + VM + while→bytecode→native → `bc_one_out.exe` через `gcc`/`cl` |

```bash
kenga run --lite kenga/compiler/more.kenga
bootstrap/bin/kenga-lite.exe run kenga/emit/lower_c.kenga
bootstrap/bin/kenga-lite.exe run kenga/emit/lower_kv.kenga
bootstrap/bin/kenga-lite.exe run kenga/emit/bc_compile_c.kenga
scripts/freedom-smoke.cmd                # more + emit → C → native
scripts/bc-run.cmd                       # .kenga → bc_one_out.exe (gcc/cl)
```

До полного chicken-egg: см. **`docs/INDEPENDENCE.md`**.  
Codegen без Rust: `lower_c` / `lower_kv` / `rt_kval` / `bc_compile_c` / `bc_src_c`.

## emit-c: tagged KVal

`kenga emit-c` / `kenga build` используют **tagged `KVal`** runtime (i64 / f64 / str / list-handle), как lite VM:

- `list` → `int64_t` handle в глобальный heap (`klist_new`, `klist_push_val`, …)
- индекс списка → `KVal`; в арифметике/сравнениях — `kval_as_i64` / `kval_as_f64` / `kval_as_list` / `kval_as_str`
- nested / hetero lists: `[[1,2], 3]`, строки в списках — ок
- `f64` / `round` — нативные `double` + `llround`

Пример: `examples/selfhost/nested_lists.kenga`.
