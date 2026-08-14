# Bootstrap 1.4

## Свежее

- Chat: «что будет завтра?» больше не путается с help (`?`)
- Self-host mini: переменные `x=2+3; y=x*4; y` на чистом Kenga
- Builtins: `to_str`, `input`, `ord`
- Self-host if/cmp: `examples/selfhost/iffy.kenga`
- Deep train 55/55 → `minds/agent.km`

## Статус

| Слой | Статус |
|---|---|
| Язык + VM (Rust host) | ✅ |
| Prophet + chat | ✅ |
| Self-host arith + mini vars | ✅ |
| Полный self-host | 🚧 |

Python не нужен. Компилятор пока на Rust.
