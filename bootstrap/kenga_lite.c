/* Kenga-lite bootstrap — C99 compiler+VM, no Rust.
 *
 * Dialect:
 *   fn / let / while / if-else / return / calls
 *   i64 / f64 arith + cmp (< > <= >= == !=)
 *   println(expr);          — print i64, f64, string, i64-list, or struct
 *   "string" literals       — values (println, == / !=)
 *   i64 lists: [1,2,3], len(xs), push(xs, v), xs[i], xs[i] = v
 *   round(x) / assert(c)    — f64→i64 / die if false
 *   nested/hetero lists, import "path", ord(s), s[i] char index
 *   str + str concat, forward fn calls, true/false
 *   for x in a..b / for v in xs / break / continue
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
 *   stubs: now_ms() → 0 (until clock port)
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
#include <direct.h>
#else
#include <sys/stat.h>
#endif

enum {
  OP_CONST = 1,
  OP_LOAD,
  OP_STORE,
  OP_ADD,
  OP_SUB,
  OP_MUL,
  OP_DIV,
  OP_LT,
  OP_GT,
  OP_EQ,
  OP_JMP,
  OP_JMPF,
  OP_HALT,
  OP_LE,
  OP_CALL,
  OP_RET,
  OP_NE,
  OP_GE,
  OP_PRINTLN,
  OP_CONST_STR,
  OP_LIST_NEW,
  OP_LIST_PUSH,
  OP_LEN,
  OP_GET,
  OP_SET,
  OP_STRUCT_NEW,
  OP_GET_FIELD,
  OP_SET_FIELD,
  OP_CONST_F64,
  OP_ROUND,
  OP_ASSERT,
  OP_POP,
  OP_ORD,
  OP_MEM_CONFIG,
  OP_MEM_REMEMBER,
  OP_MEM_SURPRISE,
  OP_MEM_FORESEE,
  OP_MEM_CONSOLIDATE,
  OP_MEM_STATS,
  OP_MEM_RECALL,
  OP_SAVE_MIND,
  OP_LOAD_MIND,
  OP_TENSOR,
  OP_T_FROM,
  OP_T_FILL,
  OP_T_GET,
  OP_T_SET,
  OP_T_SHAPE,
  OP_T_ADD,
  OP_T_SUB,
  OP_T_MUL,
  OP_T_MATMUL,
  OP_T_RESHAPE,
  OP_T_TRANSPOSE,
  OP_T_SCALE,
  OP_T_SUM,
  OP_T_DOT,
  OP_T_EXP,
  OP_T_SOFTMAX,
  OP_SWEEP,
  OP_NOW_MS,
  OP_LOAD_PPM,
  OP_LOAD_WAV,
  OP_T_MEAN,
  OP_LEARN,
  OP_PREDICT,
  OP_UNROLL,
  OP_REMEMBER_NEXT,
  OP_EMIT,
  OP_PUMP,
  OP_PENDING,
  OP_LISTEN,
  OP_TYPEOF,
  OP_AG_CLEAR,
  OP_AG_PARAM,
  OP_AG_CONST,
  OP_AG_ADD,
  OP_AG_SUB,
  OP_AG_MUL,
  OP_AG_MATMUL,
  OP_AG_SCALE,
  OP_AG_RELU,
  OP_AG_NEG,
  OP_AG_TRANSPOSE,
  OP_AG_RESHAPE,
  OP_AG_EXP,
  OP_AG_SOFTMAX,
  OP_AG_MSE,
  OP_AG_SUM,
  OP_AG_VALUE,
  OP_AG_GRAD,
  OP_AG_BACKWARD,
  OP_AG_STEP,
  OP_T_MSE,
  OP_T_PATCH_MEAN,
  OP_T_LINEAR_GRAD,
  OP_T_LOG,
  OP_AG_LOG,
  OP_SAVE_TENSOR,
  OP_LOAD_TENSOR,
  OP_WRITE_FILE,
  OP_READ_FILE
};

enum { TAG_I64 = 0, TAG_STR = 1, TAG_LIST = 2, TAG_STRUCT = 3, TAG_F64 = 4, TAG_MEMORY = 5,
       TAG_TENSOR = 6 };

typedef struct {
  int tag;
  int64_t payload; /* i64 value, or str/list/struct handle */
} Value;

typedef struct {
  int64_t *data;
  size_t len, cap;
} I64A;

typedef struct {
  Value *data;
  size_t len, cap;
} ValA;

typedef struct {
  char **data;
  size_t len, cap;
} StrA;

typedef struct {
  I64A *data;
  size_t len, cap;
} I64AA; /* array of int arrays (param lists) */

typedef struct {
  ValA *data;
  size_t len, cap;
} ListHeap;

typedef struct {
  char *name;
  StrA fields;
} StructType;

typedef struct {
  int type_id;
  I64A fields; /* i64 field values */
} StructObj;

typedef struct {
  StructObj *data;
  size_t len, cap;
} StructHeap;

typedef struct {
  char *event;
  int64_t fn_index; /* compile: fn table index; then overwritten with code addr */
  int arity;
} EvBind;

typedef struct {
  char *name;
  int64_t addr;
  int arity;
} FnInfo;

typedef struct {
  I64A code;
  StrA strs;
  StructType *stypes;
  size_t nstypes;
  EvBind *binds;
  size_t nbinds;
  FnInfo *fns;
  size_t nfns;
} Program;

static StrA *g_strtab; /* active during compile */
static StructType *g_stypes;
static size_t g_nstypes, g_stypes_cap;

#include "generated/rt_mem.inc.c"
#include "generated/rt_val.inc.c"
#include "generated/rt_arena.inc.c"

#include "prophet_lite.inc.c"
#include "tensor_lite.inc.c"
#include "tape_lite.inc.c"
#include "events_lite.inc.c"
#include "chat_lite.inc.c"

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
