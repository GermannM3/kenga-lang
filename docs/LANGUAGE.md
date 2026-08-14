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

## Living memory

```kenga
let flash: Tensor ttl 5s = tensor(8, 8);
// … использование …
sweep(); // выкинуть просроченные слоты
```

Значение с `ttl` становится недоступным после дедлайна (ошибка `expired`).
