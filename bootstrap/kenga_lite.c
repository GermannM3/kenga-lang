/* Kenga-lite bootstrap — C99 compiler+VM, no Rust.
 *
 * Dialect:
 *   fn / let / while / if-else / return / calls
 *   i64 / f64 arith + cmp (< > <= >= == !=) + && || ! + i64 %
 *   println(expr); print(expr);  — with / without newline
 *   sleep_ms(n)             — wall-clock sleep, n >= 0
 *   "string" literals       — values (println, == / !=)
 *   i64 lists: [1,2,3], len(xs), push(xs, v), xs[i], xs[i] = v
 *   round(x) / assert(c)    — f64→i64 / die if false
 *   nested/hetero lists, import "path", ord(s), s[i] char index
 *   str + str concat, forward fn calls, true/false
 *   for x in a..b / for v in xs / break / continue
 *   comments: // line  and  slash-star block comments
 *   type annotations ignored: `let x: i64 =`, `fn f(a: i64) -> i64`, `ttl 5s`
 *   Tensor (no Rust): tensor / t_from / t_fill / t_get / t_set / t_shape /
 *     t_add / t_sub / t_mul / t_matmul / t_reshape / t_transpose /
 *     t_scale / t_sum / t_dot / t_exp / t_log / t_softmax / t_mean /
 *     t_mse / t_patch_mean / t_linear_grad /
 *     save_tensor / load_tensor / write_file / read_file /
 *     load_ppm / load_wav
 *   Events: on "e"(x) { } / emit / pump / pending / listen
 *   Tape: ag_clear / ag_param / ag_const / ag_add|sub|mul|matmul /
 *     ag_scale / ag_relu / ag_neg / ag_transpose / ag_reshape /
 *     ag_exp / ag_log / ag_softmax / ag_mse / ag_sum / ag_value / ag_grad /
 *     ag_backward / ag_step
 *   now_ms() wall clock; to_str(x)
 *   structs: struct Point { x, y } / Point { x: 1, y: 2 } / p.x / p.x = v
 *
 * Usage:
 *   kenga-lite              # self-tests
 *   kenga-lite run file.kenga
 *   kenga-lite eval 'fn main(){ 2+3 }'
 */
#include <ctype.h>
#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#ifdef _WIN32
#include <windows.h>
#else
#include <sys/time.h>
#include <unistd.h>
#endif
#ifdef _WIN32
#include <direct.h>
#else
#include <sys/stat.h>
#endif

#include "generated/rt_types.inc.c"


#include "generated/rt_mem.inc.c"
#include "generated/rt_val.inc.c"
#include "generated/rt_arena.inc.c"

#include "generated/rt_prophet.inc.c"
#include "generated/rt_tensor.inc.c"
#include "generated/rt_tape.inc.c"
#include "generated/rt_events.inc.c"
#include "generated/rt_chat.inc.c"

#include "generated/rt_prog.inc.c"

#include "generated/rt_lex.inc.c"
#include "generated/rt_parse.inc.c"
#include "generated/rt_loop.inc.c"
#include "generated/rt_scan.inc.c"

static int g_tmpn = 0;

/* forward */
static size_t emit_cmp(const char *s, size_t i, size_t n, I64A *code, StrA *vnames,
                       StrA *fnames, I64A *faddrs, I64A *fargc);
static size_t emit_block(const char *s, size_t i, size_t n, I64A *code, StrA *vnames,
                         StrA *fnames, I64A *faddrs, I64A *fargc);
static size_t emit_stmt(const char *s, size_t i, size_t n, I64A *code, StrA *vnames,
                        StrA *fnames, I64A *faddrs, I64A *fargc);

#include "generated/rt_factor.inc.c"


#include "generated/rt_expr.inc.c"

#include "generated/rt_stmt.inc.c"


#include "generated/rt_compile.inc.c"


#include "generated/rt_print.inc.c"


#include "generated/rt_vm.inc.c"
#include "generated/rt_host.inc.c"
#include "generated/rt_selftest.inc.c"


#include "generated/rt_cli.inc.c"
