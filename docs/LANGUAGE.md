# Язык Kenga (кратко)

## Синтаксис

```kenga
import "stdlib/math.kenga";

struct User { id: i64, name: str }

fn greet(u: User) {
    println("hi " + u.name);
}

fn main() -> i64 {
    let xs: list = [10, 20, 30];
    xs[1] = 99;

    for i in 0..3 {
        if i == 1 { continue; }
        println(xs[i]);
    }

    let mem: Tensor ttl 1s = tensor(4);
    sweep();
    return 0;
}
```

## Builtins

- `print` / `println`
- `len` / `push`
- `tensor` / `sweep` / `now_ms` / `sleep_ms`
- `assert` / `typeof`
- `listen` / `emit` / `pump` / `pending`
- Prophet: `memory` / `memory_config` / `remember` / `surprise` / `foresee` / `consolidate` / `recall` / `mem_stats`

## Event loop

```kenga
on "tick"(n: i64) {
    println(n);
    if n < 5 { emit("tick", n + 1); }
}

fn main() {
    emit("tick", 0);
    pump(32);
}
```

Обработчики `on` регистрируются автоматически. `pump(n)` снимает до `n` событий из очереди и вызывает хендлеры.

## Память Пророка

Два слоя:

1. **Episodic** — короткоживущий буфер сюрпризов (то, что сломало ожидание)
2. **Core** — сжатые «законы мира» с importance-lock (EWC-lite), чтобы новый опыт не стирал старый

```kenga
let mind: Memory = memory_config(10, 32, 16); // threshold=0.10

let pred = foresee(mind, obs);
let s = surprise(pred, obs);
remember(mind, obs, s);   // false, если s < threshold

consolidate(mind);         // сон: replay → core + locks
recall(mind, query, 3);    // ближайшие следы
mem_stats(mind);           // [episodic, core, locked]
```

`i64` surprise в `remember` / `memory_config` трактуется как проценты (`80` → `0.80`).

## Living memory (TTL)

```kenga
let flash: Tensor ttl 5s = tensor(8, 8);
sweep(); // выкинуть просроченные слоты
```

Значение с `ttl` становится недоступным после дедлайна (ошибка `expired`).
