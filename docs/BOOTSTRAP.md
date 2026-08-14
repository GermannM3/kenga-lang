# Bootstrap 1.1

Закрытый объём «языка, на котором уже можно жить»:

| Слой | Статус |
|---|---|
| Lexer / parser / AST | ✅ |
| Bytecode + VM | ✅ |
| Living `ttl` / `sweep` | ✅ |
| Events `on` / `emit` / `pump` | ✅ |
| Prophet memory + residual MLP world-model | ✅ |
| Pure-Kenga neuromodel (`examples/neuromodel.kenga`) | ✅ |
| `emit-c` (i64, list, struct, control flow, import) | ✅ |
| `kenga build` (системный gcc/clang/cl) | ✅ |
| Stdlib `math` / `list` / `agent` | ✅ |
| CI + GitHub Releases | ✅ |

Не в 1.x (следующий горизонт): LLVM, self-host, пакетный менеджер, отдельный сайт.
