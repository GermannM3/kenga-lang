# kf_rt.h — freestanding runtime declarations for KengaOS kernels

When you build a Kenga program with `kenga emit-c <file.kenga> --freestanding`,
the generated C file pulls in **RUNTIME_FS** (a freestanding runtime) instead of the
default libc-using runtime.

`RUNTIME_FS` declares a small set of **weakly linked hooks** that a real kernel is
expected to supply. If you ship the generated `.c` without these hooks and without libc,
`k_die()` is reached on the first allocation or print.

## What the freestanding runtime gives you

- Tagged `KVal` (str / list / i64 / f64) with `klist_*`, `kstr_*`, `kval_*` helpers.
- `k_assert`, `k_ord`, `k_die`, `abort`, `exit` — `k_die` falls into `cli; hlt` loop.
- MMIO intrinsics `_k_mmio_r8/16/32/64`, `_k_mmio_w8/16/32/64` (volatile, type-safe, via `K_MMIO_R/W` macros).
- `__atomic_*` operations (SEQ_CST) — `atomic_load`, `atomic_store`, `atomic_cas`, `atomic_fence`.
- Inline asm: `asm_hlt()`, `asm_cli()`, `asm_sti()`, `asm(code)`.
- Bump allocator `_k_arena_alloc(n)` with weak fallback chain (see below).

## Weak hooks the kernel must supply

| Symbol | Signature | Default behaviour | When to override |
|---|---|---|---|
| `kf_alloc(size_t)` | `void* kf_alloc(size_t n)` | None (NULL → `k_die`) | Always in a kernel — implement on top of `kmalloc` / buddy / boot-time `sbrk`. |
| `__builtin_malloc` | `void* __builtin_malloc(size_t)` | NULL | Provided by libc (libc builds). Skipped if weak-resolved `&__builtin_malloc` is false. |

Lookup chain in `_k_arena_alloc`:

1. `kf_alloc(n)` — kernel-defined weak hook. Wins if it returns non-NULL.
2. `__builtin_malloc(n)` — libc fallback.
3. `k_die("oom")` — last resort.

## Boot data

The runtime does **not** auto-grab the stivale2 boot info. The start.S is responsible
for stashing the pointer into a kernel-visible global (e.g. `g_boot_info`), and the
generated FFIs reach it via `extern void* g_boot_info;` plus a thin C wrapper declared in
`kf_rt.h`:

```c
/* kf_rt.h — kernel-provided */
extern void* g_boot_info;
static inline void* kf_get_boot_info(void) { return g_boot_info; }
```

The Kenga source then reads it (e.g. `kf_get_boot_info()`) and the codegen emits
ordinary C calls; no special runtime support is needed.

## Minimal kernel skeleton

```c
#include <stdint.h>
#include "kf_rt.h"   /* kf_alloc, kf_get_boot_info, etc. */

/* boot.S wrote rdi (=stivale2 struct ptr) here: */
void* g_boot_info = 0;

extern void* kf_alloc(size_t);
void* kf_alloc(size_t n) {
    /* bump from a fixed heap region, or call into your real allocator. */
    static char heap[1 << 20];
    static size_t off = 0;
    if (off + n > sizeof(heap)) return 0;            /* NULL → k_die("oom") */
    void* p = heap + off;
    off += (n + 15) & ~(size_t)15;
    return p;
}

/* …your bar(), kf_framebuffer_blit(), kf_get_memmap(), etc. */
```

Link the kernel with `start.o main.o` plus `-no-pie -nostdlib -lgcc`. The runtime
auto-detects whether each weak hook is resolvable; stub it once, everything just works.

## Generated-output checklist

After `kenga emit-c my_kernel.kenga --freestanding -o my_kernel.c`:

- `my_kernel.c` includes `RUNTIME_FS` — no `<stdio.h>` / `<stdlib.h>` calls.
- Allocations go through `_k_arena_alloc` → `kf_alloc`.
- `abort`, `exit`, `assert(false)` all funnel into `k_die` (halt loop).
- All MMIO / `asm_*` / `atomic_*` are valid C — pass `clang` / `gcc --target=x86_64-elf`.
