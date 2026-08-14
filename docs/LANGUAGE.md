# Язык Kenga (кратко)

## Builtins

- `print` / `println`
- `len` / `push`
- `tensor` / `sweep` / `now_ms` / `sleep_ms`
- `assert` / `typeof`
- `listen` / `emit` / `pump` / `pending`
- Prophet memory:
  - `memory` / `memory_config`
  - `remember` / `remember_next`
  - `surprise` / `foresee` / `predict` / `learn`
  - `consolidate` / `recall` / `mem_stats`

`mem_stats` → `[episodic, core, locked, model_steps, model_dim]`

## Память Пророка + веса

```kenga
let mind = memory();

// переход obs -> next с surprise-гейтом
remember_next(mind, [1, 2, 3], [2, 3, 4], 50);

// явный шаг обучения world-model (tanh(Wx+b), EWC-locks)
learn(mind, [1, 2, 3], [2, 3, 4]);

consolidate(mind); // сон: core-fold + replay в веса

predict(mind, [1, 2, 3]); // только нейросеть
foresee(mind, [1, 2, 3]); // hybrid: сеть + core traces
```

## Event loop

```kenga
on "tick"(n: i64) {
    if n < 5 { emit("tick", n + 1); }
}
fn main() { emit("tick", 0); pump(32); }
```

## Native backend (MVP)

```bash
kenga emit-c examples/native_hello.kenga -o hello.c
# gcc hello.c -o hello && ./hello
```

Поддерживается подмножество: `i64`, `let`, `if/else`, `while`, `println`, арифметика.
