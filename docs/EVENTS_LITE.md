# Events на kenga-lite (без Rust)

С **3.5**: `on` / `emit` / `pump` / `pending` / `listen` в C99 bootstrap.

```bat
bootstrap\bin\kenga-lite.exe run examples\agent.kenga
kenga run --lite examples\agent.kenga
```

## Синтаксис

```kenga
on "sense"(x: i64) {
    emit("think", x + 1);
}

fn main() -> i64 {
    emit("sense", 0);
    let n = pump(32);
    assert(pending() == 0);
    return 0;
}
```

| Вызов | Смысл |
|---|---|
| `on "e"(args) { }` | handler (arity 0 или 1) |
| `emit("e", v)` | в очередь |
| `pump(n)` | обработать до n событий → сколько сделано |
| `pending()` | длина очереди |
| `listen("e", "fnName")` | ручная привязка к `fn` |

Демо: `examples/agent.kenga`, `examples/prophet_loop.kenga` (нужен `memory`/`typeof` — тоже на lite).
