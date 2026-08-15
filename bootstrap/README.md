# Bootstrap — свобода от Rust (Kenga-lite)

Настоящий `kenga run` пока на Rust.  
**Lite-диалект** уже компилируется **без Rust**: чистый C99 → `bootstrap/bin/kenga-lite`.

## Собрать

```bat
bootstrap\build.cmd
```

```bash
bash bootstrap/build.sh
```

Нужен MSVC / gcc / clang / `cc`.

## Запуск

```bash
./bootstrap/bin/kenga-lite run examples/selfhost/fact_lite.kenga
./bootstrap/bin/kenga-lite run examples/hello.kenga
kenga run --lite examples/hello.kenga
```

## Диалект

- `fn` / `let` / `while` / `if-else` / `return` / вызовы
- i64 арифметика и сравнения: `<` `>` `<=` `>=` `==` `!=`
- `println(expr);` — печатает i64, строку, i64-список или struct (`Point{3, 4}`)
- строковые литералы `"hello"` (println, сравнение `==` / `!=`)
- i64-списки: `[1,2,3]`, `len(xs)`, `push(xs, v)`, `xs[i]`, `xs[i] = v`
- structs: `struct Point { x, y }`, литерал `Point { x: 1, y: 2 }` (поля в любом порядке), доступ `p.x`, присваивание `p.x = v`

Нет: import, Memory, вложенные списки / строки внутри списков, типы полей (пока только i64).

## Дорога к полному chicken-egg

1. ✅ C99 bootstrap lite (этот каталог)
2. ✅ struct / richer values (i64 fields)
3. Переписать bootstrap на Kenga и `emit-c` → снова C без Rust
4. Подключить как настоящий `kenga run` для растущего подмножества
