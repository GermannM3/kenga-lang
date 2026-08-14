# Язык Kenga (кратко)

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

## Native backend (`emit-c`)

Подмножество, которое уже уходит в C99:

- `i64`, `list`, строки в `println`
- `let` / assign / index assign
- `if/else`, `while`, `for` (`0..n` и `for x in list`)
- `break` / `continue`
- `len` / `push`, вызовы функций
- `println` для i64 / str / list

```bash
kenga emit-c examples/native_lists.kenga -o lists.c
gcc lists.c -o lists && ./lists
```

Пока нет: struct, Memory/Prophet, event `on`, float-математика как first-class.
