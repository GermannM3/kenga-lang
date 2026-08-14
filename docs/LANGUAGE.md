# Язык Kenga (кратко)

## Память Пророка

World model — MLP `dim → hidden → dim` с tanh и EWC-locks:

```kenga
let mind = memory();
learn(mind, x, y);
predict(mind, x);          // один шаг (сеть)
foresee(mind, x);          // hybrid: сеть + core
unroll(mind, x, 5);        // чистое нейро-будущее на 5 шагов
foresee_n(mind, x, 5);     // hybrid-будущее на 5 шагов
consolidate(mind);         // сон
```

`mem_stats` → `[episodic, core, locked, steps, dim, hidden]`

## Event loop

```kenga
on "tick"(n: i64) { if n < 5 { emit("tick", n + 1); } }
fn main() { emit("tick", 0); pump(32); }
```

## Native backend (MVP)

```bash
kenga emit-c examples/native_hello.kenga -o hello.c
```
