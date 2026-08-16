# Язык Kenga — справочник

Версия хоста: **3.13.x**. Канон без Rust: `bootstrap/` + **`kenga/`**.  
Замена `src/`: `docs/REPLACE_RUST.md`. Путь свободы: `docs/INDEPENDENCE.md`. Smoke: `scripts/freedom-smoke.cmd`.  
Kenga→C без Rust codegen: `kenga/emit/lower_c.kenga` (см. `docs/LEARN.md` §7).

## Запуск

```bash
kenga run file.kenga
kenga run --lite file.kenga   # C99 bootstrap, подмножество
kenga eval 'println(1+2)'
kenga emit-c file.kenga -o out.c
kenga build file.kenga -o app
kenga chat minds/agent.km
kenga demo
```

## Типы

| Тип | Примечание |
|---|---|
| `i64` | целое |
| `f64` | вещественное |
| `bool` | `true` / `false` |
| `str` | `"текст"` |
| `list` | `[1, 2, x]` — гетерогенный на полном VM |
| `Tensor` | плотный f64, shape + data |
| `Memory` | Prophet mind |
| `struct Name { … }` | поля |
| `void` | только у `return` без значения |

## Синтаксис

```kenga
fn name(a: i64, b: i64) -> i64 { … }
let x: i64 = 1;
let y = x + 2;
x = x + 1;
if cond { … } else { … }
while cond { … }
for i in 0..n { … }
for v in xs { … }
break; continue;
return expr;
import "path.kenga";
struct Point { x: i64, y: i64 }
let p = Point { x: 1, y: 2 };
p.x = 3;
on "tick"(n: i64) { emit("tick", n + 1); }
```

Операторы: `+ - * / %` · `== != < <= > >=` · `&& ||` · `!` · `-` (unary).  
Lite и `more.kenga` считают `&&` `||` `!` как 0/1 (оба операнда вычисляются). `%` — только `i64`.

Комментарии: `//` и `/* … */`.

## Встроенные (общее)

`print` `println` `len` `push` `assert` `typeof` `round` `ord` `to_str` `input`  
`now_ms` `sleep_ms` `sweep`  
`listen` `emit` `pump` `pending`

## Тензоры

```kenga
tensor(2, 3)                 // нули
t_from([2, 3], […])          // shape + flat data
t_get(t, i) / t_set(t, i, v)
t_shape(t) / t_fill(t, v)
t_add / t_sub / t_mul        // elementwise
t_scale(t, s) / t_sum(t)
t_matmul(a, b)               // rank-2
t_dot(u, v)                  // rank-1 → f64
t_reshape(t, […])
t_mean(t)                    // spatial / global mean
t_sgd_step(w, x, y, lr)      // один шаг MSE для W@x≈y
load_ppm("a.ppm")            // → [h,w,3] в 0..1
load_wav("a.wav")            // PCM16 → [n] в -1..1
```

## Prophet / Memory

```kenga
let mind = memory();
// memory_config(dim, threshold, …) — см. примеры
learn(mind, x, y);
predict(mind, x);
unroll(mind, x, n);
foresee / foresee_n / surprise
remember / remember_next / recall
consolidate(mind);
mem_stats(mind);
save_mind(mind, "minds/x.km");
load_mind("minds/x.km");
```

World-model: residual MLP `y = x + Δ`.

## Lite (без Rust)

Поддерживает: `fn` `let` `while` **`for` / `break` / `continue`** `if`/`else`/`else if`,  
i64, f64, `round`, `assert`, строки, списки, `print`/`println`, `struct`, type annotations (игнор),  
**Prophet Memory** + **Tensor** (`tensor` / `t_from` / `t_matmul` / `t_add` / `load_ppm` / `load_wav` / …),
tape `ag_*`, `sweep`, `now_ms` wall clock, `sleep_ms`, `foresee_n`.

Тот же диалект на VM **`more.kenga`**: birth→24, `print`/`sleep_ms`, `learn`/`predict`/`unroll`/`remember_next`/`foresee_n`, `examples/prophet.kenga`, `mlp_autograd.kenga`, `examples/ml/kenga_net.kenga`, `examples/selfhost/vision_more.kenga`, `examples/ml/living_multimodal.kenga`, `examples/ml/word_lm.kenga`.

```bash
bash bootstrap/build.sh
kenga run --lite examples/hello.kenga
kenga run --lite examples/prophet.kenga
kenga run --lite examples/ml/tensor_core.kenga
kenga chat --lite minds/multi.km
kenga run --lite examples/native_lists.kenga
kenga run --lite examples/agent.kenga
kenga run --lite examples/selfhost/for_lite.kenga
```

Пока на полном VM (Rust): GPU / production-scale path. Tape на lite: `docs/TAPE_LITE.md`.

## emit-c / build

`lower_c` → C99: i64, f64, list, struct, for/if, events, `println`/`print`, `round`, `typeof`/`to_str`, `&&` `||` `!` `%`, `sleep_ms`, `now_ms`.  
`lower_kv` → tagged KVal C, тот же диалект + str/ord + `now_ms` + Memory + Tensor + tape (`ag_matmul`/`ag_add`/`ag_relu`/`ag_backward`/`ag_step`).  
`bc_src_c` → bytecode C VM: то же плюс birth/net/agent + `now_ms`.

## Примеры по темам

| Тема | Файл |
|---|---|
| Hello | `examples/hello.kenga` |
| Showcase | `examples/showcase.kenga` |
| Struct | `examples/native_struct.kenga` |
| World-model | `examples/ml/world_model.kenga` |
| Тензоры | `examples/ml/tensor_core.kenga` |
| SGD | `examples/ml/train_sgd.kenga` |
| Autograd tape | `examples/ml/autograd_tape.kenga` |
| MLP + tape | `examples/ml/mlp_autograd.kenga` |
| Softmax tape | `examples/ml/softmax_tape.kenga` |
| else if / transpose | `examples/control_elif.kenga` |
| Vision | `examples/ml/vision_ppm.kenga` |
| Fusion | `examples/ml/fusion.kenga` |
| Prophet | `examples/prophet.kenga` (и на `--lite`) |
| Lite | `examples/selfhost/*_lite.kenga` |
| more tensors / tape / vision | `examples/selfhost/tensor_more.kenga`, `tape_more.kenga`, `vision_more.kenga` |
| more print / learn / unroll | `examples/selfhost/print_more.kenga`, `learn_more.kenga`, `unroll_more.kenga`, `foresee_n_more.kenga` |
| more Prophet | `examples/selfhost/prophet_more.kenga`, `examples/prophet.kenga` |

Учить по шагам: `docs/LEARN.md` · упражнения: `docs/EXERCISES.md`.
