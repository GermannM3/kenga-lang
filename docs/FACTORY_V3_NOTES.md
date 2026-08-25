# Factory v3 Notes — новые конструкции языка vs токенизатор/кодек

Аудит пайплайна генерации программ (`tools/corpus_factory.py`, кодек
`minds/kenga_full.pkl`, лексеры `tools/train_m3.py` / `tools/kenchat.py`).

## Текущее состояние кодека

`minds/kenga_full.pkl` — 128 токенов: синтаксис
`: , ; { } ( ) -> + - * / = == < <= >`, ключевые слова
`fn return let if else while for i64 println`, спецтокен `ID`,
цифры `0-9`, буквы `a-z A-Z`, `_` и 38 BPE-merges
(`st en se li co t_ am list rt es as nam in de asse assert ar names code or len str ro re id lo at al ip ac ad sh em par pu sr src push`).

Ключевой факт: любое слово вне `KEYWORDS` кодируется по буквам через
`encode_word` с жадным применением merges (`tools/train_m3.py:91`,
`tools/kenchat.py:load_codec_vocab`). Поэтому новые слова уже представимы:

| Слово | Кодирование | Стоимость |
|---|---|---|
| `enum` | merge `e+n` → `en`,`u`,`m` | 3 |
| `match` | `m`,`a`,`t`,`c`,`h` | 5 |
| `union` | `u`,`n`,`i`,`o`,`n` | 5 |
| `impl`  | `i`,`m`,`p`,`l` | 4 |
| `const` | `co`,`n`,`s`,`t` | 4 |
| `Some`/`None` | по буквам | 4 |
| `step`  | `s`,`t`,`e`,`p` | 4 |
| `as`    | merge `a+s` → `as` | 1 |
| `_`     | одиночный символ vocab | 1 |

Вывод: терминалы для слов — вопрос компактности, а не корректности.

## Критические пробелы лексера (корректность!)

Сканнер вынесен в `tools/kenga_lex.py` (`kenchat.tokenize`, `train_m3.make_codec_tokenize`).
`lex_raw` уже различает `.` / `..` / `=>` / `::` / `&|^~` и больше не мапит их в `ID`.
В vocab 128 этих глифов нет: encode их **пропускает**. `0..10` в id-потоке
по-прежнему `0 1 0`. Чтобы диапазон и match жили в модели — новые терминалы
и переобучение. Тест: `python tools/test_lexer_p0.py`.

### P0 — было (осталось дырой vocab, не лексера)

1. **`.` / `..`** — лексер держит; без терминала в vocab диапазон в id-потоке не виден.
2. **`&` `|` `^` `~`** — больше не `ID`; глифов в vocab нет, encode skip.
3. **`=>`** — один op, не `=` `>`. Encode skip до терминала.
4. **`::`** — один op, не два `:`. Encode skip до терминала.

### P1 — работает, но шумно

| Конструкция | Сейчас | Комментарий |
|---|---|---|
| match arms | нужен `=>`; `match` = 5 токенов буквами | главный кандидат v3 |
| enum payload | весь синтаксис `{ : , }` есть | ок без терминала |
| compound `+= -= *= /=` | два токена | приемлемо |
| stepped ranges | `..` сломан (P0), затем `step` | после фикса работает |
| bounded arrays `array<i64,8>` | `<`,`>` есть | ок |
| fn pointers `ptr<fn(...)>` | spellable | вызова нет — отложить |
| casts `x as float` | `as` = 1 токен | уже дёшево |
| `Some`/`None` | 4 токена, частые | кандидаты на терминалы при переучивании |
| polling `while let Some(e) = p()` | spellable | редкий sugar — отложить |
| import alias `as M` | `as` = 1 токен | ок |
| `asm_inb/asm_outw` | буквы+merges | встроенные имена — ок |

Важно: при добавлении терминалов обновить также `VOCAB_TOKENS` /
`SYNTAX_SET` в `detokenize` (`kenchat.py`) — иначе генерация новых токенов
сломает обратную сборку текста, и переучить модели m4x/m5x (vocab 128 фиксирован).

## Блокер верификации: kenga-lite не знает новых фич

`corpus_factory.main()` верифицирует каждую программу через
`kenchat.run_via_kenga_lite` (bootstrap-бинарник). kenga-lite НЕ поддерживает
match/enum/union/array/ptr/casts/while-let — такие программы отбраковываются
как broken ещё до попадания в корпус, даже если Rust-компилятор их принимает.

Варианты:
- **v3a**: категории `match`/`enum` верифицировать через Rust VM
  (`kenga::driver` + `interpret`), например CLI-harness `src/bin/factory_check.rs`;
  kenga-lite оставить для старых категорий.
- **v3b**: сначала дотащить match/enum в `examples/selfhost/kenga_lite.kenga`
  (в Rust-VM lowering match тоже отсутствует — `compiler.rs:328`), тогда
  единая верификация сохранится. Дольше, но чище.

До решения не включать match/enum в корпус: Factory начнёт тиражировать
неверифицируемые программы.

## Приоритеты Factory v3

1. **Фиксы лексера P0** (`..`, `=>`, ложный маппинг `& | ^ ~` в `ID`,
   терминал `::`). Без этого любой корпус с новыми фичами размечается с
   ошибками — ranges сейчас молча разваливаются.
2. **`gen_match` + `gen_enum`** как первые новые категории: дают структурное
   ветвление и binding (аналог `bind`); требует решения по верификации (v3a/v3b)
   и терминала `=>`.
3. **Отложить**: compound-терминалы (`+=` и пр.), `Some`/`None`-терминалы,
   fn pointers, while-let, impl/ссылки, asm_* — низкая частота против цены
   переучивания кодека и моделей.
