# Язык Kenga (кратко)

Bootstrap **1.0**: VM + `emit-c` + `kenga build`.

## Память Пророка

```kenga
learn(mind, x, y);
predict(mind, x);
unroll(mind, x, 5);
foresee_n(mind, x, 5);
consolidate(mind);
```

`mem_stats` → `[episodic, core, locked, steps, dim, hidden]`

## Event loop

```kenga
on "tick"(n: i64) { if n < 5 { emit("tick", n + 1); } }
fn main() { emit("tick", 0); pump(32); }
```

## Native backend (`emit-c` / `build`)

Подмножество в C99:

- `i64`, `list`, `struct` (поля get/set), строки в `println`
- `let` / assign / index assign / field assign
- `if/else`, `while`, `for` (`0..n` и `for x in list`)
- `break` / `continue`
- `len` / `push`, вызовы функций, `import` (мержится в один Program)
- `println` для i64 / str / list

```bash
kenga emit-c examples/native_struct.kenga -o ns.c
kenga build examples/native_lists.kenga -o native_lists
# нужен gcc / clang / MSVC cl
```

Пока нет в C: Memory/Prophet, event `on`, Tensor, float как first-class.
