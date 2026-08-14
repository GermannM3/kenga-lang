<p align="center">
  <img src="assets/banner.jpg" alt="Kenga" width="640"/>
</p>

<p align="center">
  <strong>Kenga</strong> — язык программирования для живого ИИ<br/>
  память · предсказание · агенты · локальный инференс
</p>

<p align="center">
  <a href="https://github.com/GermannM3/kenga-lang/actions/workflows/ci.yml"><img src="https://github.com/GermannM3/kenga-lang/actions/workflows/ci.yml/badge.svg" alt="CI"/></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-2ee6d6?labelColor=12151a" alt="MIT"/></a>
  <a href="https://github.com/GermannM3/kenga-lang/releases"><img src="https://img.shields.io/github/v/release/GermannM3/kenga-lang?include_prereleases&color=5b9dff&labelColor=12151a" alt="release"/></a>
  <img src="https://img.shields.io/badge/friends--ready-2.0-5b9dff?labelColor=12151a" alt="friends-ready"/>
</p>

---

## Для знакомых (30 секунд)

```bash
git clone https://github.com/GermannM3/kenga-lang.git
cd kenga-lang
cargo install --path . --force --locked
kenga demo
```

Потом: `kenga chat minds/agent.km` → «смотри 5 1 6», «что будет завтра?»

Подробный питч и честные «да/нет»: **[docs/FOR_FRIENDS.md](docs/FOR_FRIENDS.md)**

---

## Зачем Kenga

Обычные стеки для ML — Python-клей вокруг C++/CUDA.  
**Kenga** — язык, где тензор, `ttl`, консолидация и агентный цикл часть семантики.

Сейчас — **friends-ready 2.0**: рабочий язык + Prophet world-model + chat + self-host ladder (Kenga-lite на чистой Kenga).  
Rust — временный хост компилятора (как C у раннего Go). **Python не нужен.**

<p align="center">
  <img src="assets/architecture.jpg" alt="Kenga architecture" width="900"/>
</p>

### Честно

| Да | Нет |
|---|---|
| Свой язык + VM + living memory | Не ChatGPT/LLM из коробки |
| Residual MLP world-model | Не замена PyTorch |
| Self-host ladder уже бежит | Полный chicken-egg ещё нет |

---

## Установка

Нужен [Rust](https://rustup.rs/) (1.75+).

```bash
git clone https://github.com/GermannM3/kenga-lang.git
cd kenga-lang
cargo install --path . --force --locked
kenga version   # kenga 2.0.0 (friends-ready)
```

Или бинарники с [Releases](https://github.com/GermannM3/kenga-lang/releases) — но для `kenga demo` нужен клон с `examples/`.

---

## Быстрый старт

```bash
kenga demo
kenga run examples/ml/world_model.kenga
kenga run examples/ml/surprise_gate.kenga
kenga run examples/neuromodel.kenga
kenga chat minds/agent.km
kenga run examples/selfhost/kenga_lite.kenga
kenga about
```

Минимальная программа:

```kenga
fn main() -> i64 {
    println("hello from kenga");
    return 0;
}
```

World-model:

```kenga
fn main() {
    let mind = memory();
    learn(mind, [1, 0, 0], [0, 1, 0]);
    println(round(predict(mind, [1, 0, 0])));
    println(unroll(mind, [1, 0, 0], 3));
}
```

---

## Что уже есть

| Возможность | Статус |
|---|---|
| Функции, if/else, while, for/in, break/continue | ✅ |
| Списки, ranges, struct, import | ✅ |
| Living `ttl` / `sweep`, events on/emit/pump | ✅ |
| Prophet memory + residual MLP + unroll | ✅ |
| `kenga chat` (русский диалог с mind) | ✅ |
| `save_mind` / `load_mind` | ✅ |
| `kenga demo` тур для друзей | ✅ |
| emit-c / build | ✅ |
| Self-host Kenga-lite → bytecode VM | ✅ |
| Полный self-host / LLVM / LLM | 🚧 |

---

## Команды CLI

```
kenga demo|tour
kenga about | which | version
kenga run <file.kenga>
kenga chat [mind.km] [--script f]
kenga eval | parse | compile | emit-c | build
```

---

## Структура

```
kenga-lang/
├── src/              # bootstrap compiler + VM + chat
├── examples/         # demos + ml/ + selfhost/
├── minds/            # сохранённые world-model
├── stdlib/
├── docs/             # FOR_FRIENDS, LANGUAGE, SELFHOST
└── .github/          # CI + releases
```

Связанные проекты: [KengaAI_Engine](https://github.com/GermannM3/KengaAI_Engine), [The-Prophet](https://github.com/GermannM3/The-Prophet), [kengarust](https://github.com/GermannM3/kengarust).

---

## Лицензия

[MIT](LICENSE) © Kenga AI / [GermannM3](https://github.com/GermannM3)
