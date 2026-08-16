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

static size_t emit_factor(const char *s, size_t i, size_t n, I64A *code, StrA *vnames,
                         StrA *fnames, I64A *faddrs, I64A *fargc) {
  i = skip(s, i, n);
  if (i >= n) die("unexpected eof in factor");

  if (s[i] == '(') {
    i = emit_cmp(s, i + 1, n, code, vnames, fnames, faddrs, fargc);
    i = skip(s, i, n);
    if (i >= n || s[i] != ')') die("expected )");
    i = i + 1;
  } else if (s[i] == '-') {
    emit2(code, OP_CONST, 0);
    i = emit_factor(s, i + 1, n, code, vnames, fnames, faddrs, fargc);
    i64a_push(code, OP_SUB);
    return i; /* unary-minus already includes postfix via recurse */
  } else if (s[i] == '"') {
    size_t ii = i;
    char *lit = parse_string(s, &ii, n);
    int idx = intern_str(lit);
    free(lit);
    emit2(code, OP_CONST_STR, idx);
    i = ii;
  } else if (s[i] == '[') {
    i++;
    i64a_push(code, OP_LIST_NEW);
    i = skip(s, i, n);
    if (i < n && s[i] != ']') {
      for (;;) {
        i = emit_cmp(s, i, n, code, vnames, fnames, faddrs, fargc);
        i64a_push(code, OP_LIST_PUSH);
        i = skip(s, i, n);
        if (i < n && s[i] == ',') {
          i++;
          continue;
        }
        break;
      }
    }
    i = skip(s, i, n);
    if (i >= n || s[i] != ']') die("expected ]");
    i++;
  } else if (is_ident_start(s[i])) {
    Ident id = parse_ident(s, i, n);
    if (strcmp(id.name, "true") == 0) {
      free(id.name);
      emit2(code, OP_CONST, 1);
      i = id.i;
    } else if (strcmp(id.name, "false") == 0) {
      free(id.name);
      emit2(code, OP_CONST, 0);
      i = id.i;
    } else {
    size_t j = skip(s, id.i, n);
    if (j < n && s[j] == '{') {
      /* struct literal only if name is a known struct (else `xs {` is for-body) */
      int tid = find_struct_type(id.name);
      if (tid < 0) {
        int sl = slot_of(vnames, id.name);
        free(id.name);
        emit2(code, OP_LOAD, sl);
        i = id.i;
      } else {
      free(id.name);
      StructType *st = &g_stypes[tid];
      j++;
      StrA init_names = {0};
      size_t *init_pos = NULL;
      size_t ninit = 0, cap_init = 0;
      j = skip(s, j, n);
      if (j < n && s[j] != '}') {
        for (;;) {
          Ident fn = parse_ident(s, j, n);
          j = skip(s, fn.i, n);
          if (j >= n || s[j] != ':') {
            free(fn.name);
            die("expected : in struct literal");
          }
          j++;
          size_t expr_at = j;
          size_t code_save = code->len;
          j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
          code->len = code_save; /* rewind: emit later in decl order */
          if (field_index_of(st, fn.name) < 0) {
            free(fn.name);
            die("unknown struct field");
          }
          for (size_t k = 0; k < ninit; k++) {
            if (strcmp(init_names.data[k], fn.name) == 0) {
              free(fn.name);
              die("duplicate struct field");
            }
          }
          stra_push(&init_names, fn.name);
          if (ninit + 1 > cap_init) {
            cap_init = cap_init ? cap_init * 2 : 4;
            init_pos = (size_t *)xrealloc(init_pos, cap_init * sizeof(size_t));
          }
          init_pos[ninit++] = expr_at;
          j = skip(s, j, n);
          if (j < n && s[j] == ',') {
            j++;
            j = skip(s, j, n);
            continue;
          }
          break;
        }
      }
      j = skip(s, j, n);
      if (j >= n || s[j] != '}') die("expected } in struct literal");
      j++;
      if (ninit != st->fields.len) {
        for (size_t k = 0; k < init_names.len; k++) free(init_names.data[k]);
        free(init_names.data);
        free(init_pos);
        die("struct literal missing fields");
      }
      for (size_t fi = 0; fi < st->fields.len; fi++) {
        int found = -1;
        for (size_t k = 0; k < ninit; k++) {
          if (strcmp(init_names.data[k], st->fields.data[fi]) == 0) {
            found = (int)k;
            break;
          }
        }
        if (found < 0) {
          for (size_t k = 0; k < init_names.len; k++) free(init_names.data[k]);
          free(init_names.data);
          free(init_pos);
          die("struct literal missing fields");
        }
        emit_cmp(s, init_pos[found], n, code, vnames, fnames, faddrs, fargc);
      }
      for (size_t k = 0; k < init_names.len; k++) free(init_names.data[k]);
      free(init_names.data);
      free(init_pos);
      emit2(code, OP_STRUCT_NEW, tid);
      i = j;
      } /* known struct literal */
    } else if (j < n && s[j] == '(') {
      /* builtins: len(xs), push(xs, v) */
      if (strcmp(id.name, "len") == 0) {
        free(id.name);
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ')') die("expected ) in len");
        j++;
        i64a_push(code, OP_LEN);
        i = j;
      } else if (strcmp(id.name, "push") == 0) {
        free(id.name);
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ',') die("expected , in push");
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ')') die("expected ) in push");
        j++;
        i64a_push(code, OP_LIST_PUSH);
        i = j;
      } else if (strcmp(id.name, "round") == 0) {
        free(id.name);
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ')') die("expected ) in round");
        j++;
        i64a_push(code, OP_ROUND);
        i = j;
      } else if (strcmp(id.name, "assert") == 0) {
        free(id.name);
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ')') die("expected ) in assert");
        j++;
        i64a_push(code, OP_ASSERT);
        i = j;
      } else if (strcmp(id.name, "ord") == 0) {
        free(id.name);
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ')') die("expected ) in ord");
        j++;
        i64a_push(code, OP_ORD);
        i = j;
      } else if (strcmp(id.name, "memory_config") == 0) {
        free(id.name);
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ',') die("expected , in memory_config");
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ',') die("expected , in memory_config");
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ')') die("expected ) in memory_config");
        j++;
        i64a_push(code, OP_MEM_CONFIG);
        i = j;
      } else if (strcmp(id.name, "remember") == 0) {
        free(id.name);
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ',') die("expected , in remember");
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ',') die("expected , in remember");
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ')') die("expected ) in remember");
        j++;
        i64a_push(code, OP_MEM_REMEMBER);
        i = j;
      } else if (strcmp(id.name, "surprise") == 0) {
        free(id.name);
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ',') die("expected , in surprise");
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ')') die("expected ) in surprise");
        j++;
        i64a_push(code, OP_MEM_SURPRISE);
        i = j;
      } else if (strcmp(id.name, "foresee") == 0) {
        free(id.name);
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ',') die("expected , in foresee");
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ')') die("expected ) in foresee");
        j++;
        i64a_push(code, OP_MEM_FORESEE);
        i = j;
      } else if (strcmp(id.name, "consolidate") == 0) {
        free(id.name);
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ')') die("expected ) in consolidate");
        j++;
        i64a_push(code, OP_MEM_CONSOLIDATE);
        i = j;
      } else if (strcmp(id.name, "mem_stats") == 0) {
        free(id.name);
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ')') die("expected ) in mem_stats");
        j++;
        i64a_push(code, OP_MEM_STATS);
        i = j;
      } else if (strcmp(id.name, "recall") == 0) {
        free(id.name);
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ',') die("expected , in recall");
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ',') die("expected , in recall");
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ')') die("expected ) in recall");
        j++;
        i64a_push(code, OP_MEM_RECALL);
        i = j;
      } else if (strcmp(id.name, "save_mind") == 0) {
        free(id.name);
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ',') die("expected , in save_mind");
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ')') die("expected ) in save_mind");
        j++;
        i64a_push(code, OP_SAVE_MIND);
        i = j;
      } else if (strcmp(id.name, "load_mind") == 0) {
        free(id.name);
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ')') die("expected ) in load_mind");
        j++;
        i64a_push(code, OP_LOAD_MIND);
        i = j;
      } else if (strcmp(id.name, "tensor") == 0) {
        free(id.name);
        j++;
        {
          int argc = 0;
          j = skip(s, j, n);
          if (j < n && s[j] != ')') {
            for (;;) {
              j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
              argc++;
              j = skip(s, j, n);
              if (j < n && s[j] == ',') {
                j++;
                continue;
              }
              break;
            }
          }
          if (j >= n || s[j] != ')') die("expected ) in tensor");
          j++;
          if (argc < 1 || argc > TL_RANK_MAX) die("tensor dims 1..rank_max");
          emit2(code, OP_TENSOR, argc);
        }
        i = j;
      } else if (strcmp(id.name, "t_from") == 0) {
        free(id.name);
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ',') die("expected , in t_from");
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ')') die("expected ) in t_from");
        j++;
        i64a_push(code, OP_T_FROM);
        i = j;
      } else if (strcmp(id.name, "t_fill") == 0) {
        free(id.name);
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ',') die("expected , in t_fill");
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ')') die("expected ) in t_fill");
        j++;
        i64a_push(code, OP_T_FILL);
        i = j;
      } else if (strcmp(id.name, "t_get") == 0) {
        free(id.name);
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ',') die("expected , in t_get");
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ')') die("expected ) in t_get");
        j++;
        i64a_push(code, OP_T_GET);
        i = j;
      } else if (strcmp(id.name, "t_set") == 0) {
        free(id.name);
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ',') die("expected , in t_set");
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ',') die("expected , in t_set");
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ')') die("expected ) in t_set");
        j++;
        i64a_push(code, OP_T_SET);
        i = j;
      } else if (strcmp(id.name, "t_shape") == 0) {
        free(id.name);
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ')') die("expected ) in t_shape");
        j++;
        i64a_push(code, OP_T_SHAPE);
        i = j;
      } else if (strcmp(id.name, "t_add") == 0 || strcmp(id.name, "t_sub") == 0 ||
                 strcmp(id.name, "t_mul") == 0 || strcmp(id.name, "t_matmul") == 0 ||
                 strcmp(id.name, "t_dot") == 0) {
        int64_t op = OP_T_ADD;
        if (strcmp(id.name, "t_sub") == 0) op = OP_T_SUB;
        else if (strcmp(id.name, "t_mul") == 0) op = OP_T_MUL;
        else if (strcmp(id.name, "t_matmul") == 0) op = OP_T_MATMUL;
        else if (strcmp(id.name, "t_dot") == 0) op = OP_T_DOT;
        free(id.name);
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ',') die("expected , in tensor binary");
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ')') die("expected ) in tensor binary");
        j++;
        i64a_push(code, op);
        i = j;
      } else if (strcmp(id.name, "t_reshape") == 0 || strcmp(id.name, "t_scale") == 0) {
        int64_t op = strcmp(id.name, "t_scale") == 0 ? OP_T_SCALE : OP_T_RESHAPE;
        free(id.name);
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ',') die("expected ,");
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ')') die("expected )");
        j++;
        i64a_push(code, op);
        i = j;
      } else if (strcmp(id.name, "t_transpose") == 0 || strcmp(id.name, "t_exp") == 0 ||
                 strcmp(id.name, "t_softmax") == 0 || strcmp(id.name, "t_sum") == 0 ||
                 strcmp(id.name, "t_log") == 0) {
        int64_t op = OP_T_TRANSPOSE;
        if (strcmp(id.name, "t_exp") == 0) op = OP_T_EXP;
        else if (strcmp(id.name, "t_softmax") == 0) op = OP_T_SOFTMAX;
        else if (strcmp(id.name, "t_sum") == 0) op = OP_T_SUM;
        else if (strcmp(id.name, "t_log") == 0) op = OP_T_LOG;
        free(id.name);
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ')') die("expected )");
        j++;
        i64a_push(code, op);
        i = j;
      } else if (strcmp(id.name, "t_mean") == 0 || strcmp(id.name, "load_ppm") == 0 ||
                 strcmp(id.name, "load_wav") == 0 || strcmp(id.name, "load_tensor") == 0 ||
                 strcmp(id.name, "read_file") == 0) {
        int64_t op = OP_T_MEAN;
        if (strcmp(id.name, "load_ppm") == 0) op = OP_LOAD_PPM;
        else if (strcmp(id.name, "load_wav") == 0) op = OP_LOAD_WAV;
        else if (strcmp(id.name, "load_tensor") == 0) op = OP_LOAD_TENSOR;
        else if (strcmp(id.name, "read_file") == 0) op = OP_READ_FILE;
        free(id.name);
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ')') die("expected )");
        j++;
        i64a_push(code, op);
        i = j;
      } else if (strcmp(id.name, "save_tensor") == 0 || strcmp(id.name, "write_file") == 0) {
        int64_t op = strcmp(id.name, "write_file") == 0 ? OP_WRITE_FILE : OP_SAVE_TENSOR;
        free(id.name);
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ',') die("expected ,");
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ')') die("expected )");
        j++;
        i64a_push(code, op);
        i = j;
      } else if (strcmp(id.name, "t_mse") == 0) {
        free(id.name);
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ',') die("expected , in t_mse");
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ')') die("expected ) in t_mse");
        j++;
        i64a_push(code, OP_T_MSE);
        i = j;
      } else if (strcmp(id.name, "t_patch_mean") == 0) {
        free(id.name);
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ',') die("expected , in t_patch_mean");
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ',') die("expected , in t_patch_mean");
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ')') die("expected ) in t_patch_mean");
        j++;
        i64a_push(code, OP_T_PATCH_MEAN);
        i = j;
      } else if (strcmp(id.name, "t_linear_grad") == 0) {
        free(id.name);
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ',') die("expected , in t_linear_grad");
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ',') die("expected , in t_linear_grad");
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ')') die("expected ) in t_linear_grad");
        j++;
        i64a_push(code, OP_T_LINEAR_GRAD);
        i = j;
      } else if (strcmp(id.name, "learn") == 0 || strcmp(id.name, "predict") == 0) {
        int64_t op = strcmp(id.name, "learn") == 0 ? OP_LEARN : OP_PREDICT;
        free(id.name);
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ',') die("expected ,");
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (op == OP_LEARN) {
          if (j >= n || s[j] != ',') die("expected , in learn");
          j++;
          j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
          j = skip(s, j, n);
        }
        if (j >= n || s[j] != ')') die("expected )");
        j++;
        i64a_push(code, op);
        i = j;
      } else if (strcmp(id.name, "unroll") == 0 || strcmp(id.name, "remember_next") == 0) {
        int64_t op = strcmp(id.name, "unroll") == 0 ? OP_UNROLL : OP_REMEMBER_NEXT;
        free(id.name);
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ',') die("expected ,");
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ',') die("expected ,");
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (op == OP_REMEMBER_NEXT) {
          if (j >= n || s[j] != ',') die("expected , in remember_next");
          j++;
          j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
          j = skip(s, j, n);
        }
        if (j >= n || s[j] != ')') die("expected )");
        j++;
        i64a_push(code, op);
        i = j;
      } else if (strcmp(id.name, "emit") == 0) {
        free(id.name);
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ',') die("expected , in emit");
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ')') die("expected ) in emit");
        j++;
        i64a_push(code, OP_EMIT);
        i = j;
      } else if (strcmp(id.name, "pump") == 0) {
        free(id.name);
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ')') die("expected ) in pump");
        j++;
        i64a_push(code, OP_PUMP);
        i = j;
      } else if (strcmp(id.name, "pending") == 0) {
        free(id.name);
        j++;
        j = skip(s, j, n);
        if (j >= n || s[j] != ')') die("expected ) in pending");
        j++;
        i64a_push(code, OP_PENDING);
        i = j;
      } else if (strcmp(id.name, "listen") == 0) {
        /* listen(event, handlerName) — resolve handler name at runtime via Program */
        free(id.name);
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ',') die("expected , in listen");
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ')') die("expected ) in listen");
        j++;
        i64a_push(code, OP_LISTEN);
        i = j;
      } else if (strcmp(id.name, "memory") == 0) {
        free(id.name);
        j++;
        j = skip(s, j, n);
        if (j >= n || s[j] != ')') die("expected ) in memory");
        j++;
        emit2(code, OP_CONST, 15); /* thr 0.15 */
        emit2(code, OP_CONST, 64);
        emit2(code, OP_CONST, 32);
        i64a_push(code, OP_MEM_CONFIG);
        i = j;
      } else if (strcmp(id.name, "typeof") == 0) {
        free(id.name);
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ')') die("expected ) in typeof");
        j++;
        i64a_push(code, OP_TYPEOF);
        i = j;
      } else if (strcmp(id.name, "ag_clear") == 0) {
        free(id.name);
        j++;
        j = skip(s, j, n);
        if (j >= n || s[j] != ')') die("expected ) in ag_clear");
        j++;
        i64a_push(code, OP_AG_CLEAR);
        i = j;
      } else if (strcmp(id.name, "ag_param") == 0 || strcmp(id.name, "ag_const") == 0 ||
                 strcmp(id.name, "ag_relu") == 0 || strcmp(id.name, "ag_neg") == 0 ||
                 strcmp(id.name, "ag_transpose") == 0 || strcmp(id.name, "ag_exp") == 0 ||
                 strcmp(id.name, "ag_log") == 0 ||
                 strcmp(id.name, "ag_softmax") == 0 || strcmp(id.name, "ag_sum") == 0 ||
                 strcmp(id.name, "ag_value") == 0 || strcmp(id.name, "ag_grad") == 0 ||
                 strcmp(id.name, "ag_backward") == 0) {
        int64_t op = OP_AG_PARAM;
        if (strcmp(id.name, "ag_const") == 0) op = OP_AG_CONST;
        else if (strcmp(id.name, "ag_relu") == 0) op = OP_AG_RELU;
        else if (strcmp(id.name, "ag_neg") == 0) op = OP_AG_NEG;
        else if (strcmp(id.name, "ag_transpose") == 0) op = OP_AG_TRANSPOSE;
        else if (strcmp(id.name, "ag_exp") == 0) op = OP_AG_EXP;
        else if (strcmp(id.name, "ag_log") == 0) op = OP_AG_LOG;
        else if (strcmp(id.name, "ag_softmax") == 0) op = OP_AG_SOFTMAX;
        else if (strcmp(id.name, "ag_sum") == 0) op = OP_AG_SUM;
        else if (strcmp(id.name, "ag_value") == 0) op = OP_AG_VALUE;
        else if (strcmp(id.name, "ag_grad") == 0) op = OP_AG_GRAD;
        else if (strcmp(id.name, "ag_backward") == 0) op = OP_AG_BACKWARD;
        free(id.name);
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ')') die("expected ) in ag unary");
        j++;
        i64a_push(code, op);
        i = j;
      } else if (strcmp(id.name, "ag_add") == 0 || strcmp(id.name, "ag_sub") == 0 ||
                 strcmp(id.name, "ag_mul") == 0 || strcmp(id.name, "ag_matmul") == 0 ||
                 strcmp(id.name, "ag_mse") == 0 || strcmp(id.name, "ag_scale") == 0 ||
                 strcmp(id.name, "ag_reshape") == 0 || strcmp(id.name, "ag_step") == 0) {
        int64_t op = OP_AG_ADD;
        if (strcmp(id.name, "ag_sub") == 0) op = OP_AG_SUB;
        else if (strcmp(id.name, "ag_mul") == 0) op = OP_AG_MUL;
        else if (strcmp(id.name, "ag_matmul") == 0) op = OP_AG_MATMUL;
        else if (strcmp(id.name, "ag_mse") == 0) op = OP_AG_MSE;
        else if (strcmp(id.name, "ag_scale") == 0) op = OP_AG_SCALE;
        else if (strcmp(id.name, "ag_reshape") == 0) op = OP_AG_RESHAPE;
        else if (strcmp(id.name, "ag_step") == 0) op = OP_AG_STEP;
        free(id.name);
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ',') die("expected , in ag binary");
        j++;
        j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
        j = skip(s, j, n);
        if (j >= n || s[j] != ')') die("expected ) in ag binary");
        j++;
        i64a_push(code, op);
        i = j;
      } else if (strcmp(id.name, "sweep") == 0) {
        free(id.name);
        j++;
        j = skip(s, j, n);
        if (j >= n || s[j] != ')') die("expected ) in sweep");
        j++;
        i64a_push(code, OP_SWEEP);
        i = j;
      } else if (strcmp(id.name, "now_ms") == 0) {
        free(id.name);
        j++;
        j = skip(s, j, n);
        if (j >= n || s[j] != ')') die("expected ) in now_ms");
        j++;
        i64a_push(code, OP_NOW_MS);
        i = j;
      } else if (strcmp(id.name, "println") == 0) {
        free(id.name);
        die("println is a statement, not an expression");
      } else {
        j++;
        int argc = 0;
        j = skip(s, j, n);
        if (j < n && s[j] != ')') {
          for (;;) {
            j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
            argc++;
            j = skip(s, j, n);
            if (j < n && s[j] == ',') {
              j++;
              continue;
            }
            break;
          }
        }
        if (j >= n || s[j] != ')') die("expected ) in call");
        j++;
        int found = 0;
        size_t fni = 0;
        for (size_t fi = 0; fi < fnames->len; fi++) {
          if (strcmp(fnames->data[fi], id.name) == 0) {
            found = 1;
            fni = fi;
            if (fargc->data[fi] != argc) die("argc mismatch");
          }
        }
        free(id.name);
        if (!found) die("unknown function");
        /* addr patched in call_patches_apply after all fn bodies exist */
        emit3(code, OP_CALL, 0, argc);
        call_patch_add(code->len - 2, fni);
        i = j;
      }
    } else {
      int sl = slot_of(vnames, id.name);
      free(id.name);
      emit2(code, OP_LOAD, sl);
      i = id.i;
    }
    } /* not true/false */
  } else {
    size_t ii = i;
    ParsedNum num = parse_number(s, &ii, n);
    if (num.is_float) {
      int64_t bits = 0;
      memcpy(&bits, &num.f, sizeof(double));
      emit2(code, OP_CONST_F64, bits);
    } else {
      emit2(code, OP_CONST, num.i);
    }
    i = ii;
  }

  /* postfix: .field and [index], any chain */
  for (;;) {
    i = skip(s, i, n);
    if (i >= n) break;
    if (s[i] == '.') {
      /* `..` is range, not field */
      if (i + 1 < n && s[i + 1] == '.') break;
      i++;
      Ident fld = parse_ident(s, i, n);
      emit2(code, OP_GET_FIELD, intern_str(fld.name));
      free(fld.name);
      i = fld.i;
      continue;
    }
    if (s[i] == '[') {
      i++;
      i = emit_cmp(s, i, n, code, vnames, fnames, faddrs, fargc);
      i = skip(s, i, n);
      if (i >= n || s[i] != ']') die("expected ]");
      i++;
      i64a_push(code, OP_GET);
      continue;
    }
    break;
  }
  return i;
}

#include "generated/rt_expr.inc.c"

static size_t emit_stmt(const char *s, size_t i, size_t n, I64A *code, StrA *vnames,
                        StrA *fnames, I64A *faddrs, I64A *fargc) {
  i = skip(s, i, n);
  if (starts_kw(s, i, n, "let")) {
    i = skip(s, i + 3, n);
    Ident id = parse_ident(s, i, n);
    i = skip_type_annot(s, id.i, n);
    i = skip(s, i, n);
    if (i >= n || s[i] != '=') die("expected =");
    i = emit_cmp(s, i + 1, n, code, vnames, fnames, faddrs, fargc);
    int sl = slot_of(vnames, id.name);
    free(id.name);
    emit2(code, OP_STORE, sl);
    i64a_push(code, OP_POP);
    i = skip(s, i, n);
    if (i < n && s[i] == ';') i++;
    return i;
  }
  if (starts_kw(s, i, n, "return")) {
    i = emit_cmp(s, i + 6, n, code, vnames, fnames, faddrs, fargc);
    i64a_push(code, OP_RET);
    i = skip(s, i, n);
    if (i < n && s[i] == ';') i++;
    return i;
  }
  if (starts_kw(s, i, n, "while")) {
    size_t loop_start = code->len;
    i = emit_cmp(s, i + 5, n, code, vnames, fnames, faddrs, fargc);
    size_t jmpf_at = code->len;
    emit2(code, OP_JMPF, 0);
    /* continue re-checks condition */
    loop_push(loop_start);
    i = emit_block(s, i, n, code, vnames, fnames, faddrs, fargc);
    emit2(code, OP_JMP, (int64_t)loop_start);
    code->data[jmpf_at + 1] = (int64_t)code->len;
    loop_finish(code, code->len);
    return i;
  }
  if (starts_kw(s, i, n, "for")) {
    /* for name in start..end { }  |  for name in list { } */
    i = skip(s, i + 3, n);
    Ident var = parse_ident(s, i, n);
    i = skip(s, var.i, n);
    if (!starts_kw(s, i, n, "in")) {
      free(var.name);
      die("expected in after for variable");
    }
    i = skip(s, i + 2, n);
    int vslot = slot_of(vnames, var.name);
    /* emit start or list expr */
    i = emit_cmp(s, i, n, code, vnames, fnames, faddrs, fargc);
    i = skip(s, i, n);
    if (starts_with(s, i, n, "..")) {
      /* range: for v in a..b */
      emit2(code, OP_STORE, vslot);
      i64a_push(code, OP_POP);
      i = skip(s, i + 2, n);
      char endn[32];
      snprintf(endn, sizeof endn, "__fe%d", g_tmpn++);
      int eslot = slot_of(vnames, endn);
      i = emit_cmp(s, i, n, code, vnames, fnames, faddrs, fargc);
      emit2(code, OP_STORE, eslot);
      i64a_push(code, OP_POP);
      size_t loop_start = code->len;
      emit2(code, OP_LOAD, vslot);
      emit2(code, OP_LOAD, eslot);
      i64a_push(code, OP_LT);
      size_t jmpf_at = code->len;
      emit2(code, OP_JMPF, 0);
      loop_push(0);
      i = emit_block(s, i, n, code, vnames, fnames, faddrs, fargc);
      size_t cont_at = code->len;
      g_loops[g_loop_sp].cont_target = cont_at;
      emit2(code, OP_LOAD, vslot);
      emit2(code, OP_CONST, 1);
      i64a_push(code, OP_ADD);
      emit2(code, OP_STORE, vslot);
      i64a_push(code, OP_POP);
      emit2(code, OP_JMP, (int64_t)loop_start);
      code->data[jmpf_at + 1] = (int64_t)code->len;
      loop_finish(code, code->len);
      free(var.name);
      return i;
    }
    /* list iteration */
    {
      char ln[32], ix[32];
      snprintf(ln, sizeof ln, "__fl%d", g_tmpn);
      snprintf(ix, sizeof ix, "__fi%d", g_tmpn);
      g_tmpn++;
      int lslot = slot_of(vnames, ln);
      int islot = slot_of(vnames, ix);
      emit2(code, OP_STORE, lslot);
      i64a_push(code, OP_POP);
      emit2(code, OP_CONST, 0);
      emit2(code, OP_STORE, islot);
      i64a_push(code, OP_POP);
      size_t loop_start = code->len;
      emit2(code, OP_LOAD, islot);
      emit2(code, OP_LOAD, lslot);
      i64a_push(code, OP_LEN);
      i64a_push(code, OP_LT);
      size_t jmpf_at = code->len;
      emit2(code, OP_JMPF, 0);
      emit2(code, OP_LOAD, lslot);
      emit2(code, OP_LOAD, islot);
      i64a_push(code, OP_GET);
      emit2(code, OP_STORE, vslot);
      i64a_push(code, OP_POP);
      loop_push(0);
      i = emit_block(s, i, n, code, vnames, fnames, faddrs, fargc);
      size_t cont_at = code->len;
      g_loops[g_loop_sp].cont_target = cont_at;
      emit2(code, OP_LOAD, islot);
      emit2(code, OP_CONST, 1);
      i64a_push(code, OP_ADD);
      emit2(code, OP_STORE, islot);
      i64a_push(code, OP_POP);
      emit2(code, OP_JMP, (int64_t)loop_start);
      code->data[jmpf_at + 1] = (int64_t)code->len;
      loop_finish(code, code->len);
      free(var.name);
      return i;
    }
  }
  if (starts_kw(s, i, n, "break")) {
    loop_add_break(code);
    i = skip(s, i + 5, n);
    if (i < n && s[i] == ';') i++;
    return i;
  }
  if (starts_kw(s, i, n, "continue")) {
    loop_add_continue(code);
    i = skip(s, i + 8, n);
    if (i < n && s[i] == ';') i++;
    return i;
  }
  if (starts_kw(s, i, n, "if")) {
    i = emit_cmp(s, i + 2, n, code, vnames, fnames, faddrs, fargc);
    size_t jmpf_at = code->len;
    emit2(code, OP_JMPF, 0);
    i = emit_block(s, i, n, code, vnames, fnames, faddrs, fargc);
    i = skip(s, i, n);
    if (starts_kw(s, i, n, "else")) {
      size_t jmp_at = code->len;
      emit2(code, OP_JMP, 0);
      code->data[jmpf_at + 1] = (int64_t)code->len;
      i = skip(s, i + 4, n);
      if (starts_kw(s, i, n, "if")) {
        i = emit_stmt(s, i, n, code, vnames, fnames, faddrs, fargc);
      } else {
        i = emit_block(s, i, n, code, vnames, fnames, faddrs, fargc);
      }
      code->data[jmp_at + 1] = (int64_t)code->len;
    } else {
      code->data[jmpf_at + 1] = (int64_t)code->len;
    }
    return i;
  }
  if (starts_kw(s, i, n, "println")) {
    i = skip(s, i + 7, n);
    if (i >= n || s[i] != '(') die("expected ( after println");
    i = emit_cmp(s, i + 1, n, code, vnames, fnames, faddrs, fargc);
    i = skip(s, i, n);
    if (i >= n || s[i] != ')') die("expected ) after println");
    i++;
    i64a_push(code, OP_PRINTLN);
    i = skip(s, i, n);
    if (i < n && s[i] == ';') i++;
    return i;
  }
  if (is_ident_start(s[i])) {
    size_t save = i;
    Ident id = parse_ident(s, i, n);
    size_t j = skip(s, id.i, n);
    /* xs[i] = v */
    if (j < n && s[j] == '[') {
      int sl = slot_of(vnames, id.name);
      free(id.name);
      emit2(code, OP_LOAD, sl);
      j++;
      j = emit_cmp(s, j, n, code, vnames, fnames, faddrs, fargc);
      j = skip(s, j, n);
      if (j >= n || s[j] != ']') die("expected ]");
      j = skip(s, j + 1, n);
      if (j < n && s[j] == '=' && !(j + 1 < n && s[j + 1] == '=')) {
        j = emit_cmp(s, j + 1, n, code, vnames, fnames, faddrs, fargc);
        i64a_push(code, OP_SET);
        i64a_push(code, OP_POP);
        j = skip(s, j, n);
        if (j < n && s[j] == ';') j++;
        return j;
      }
      /* not assignment — reparse as expression from save */
      if (code->len >= 2) code->len -= 2;
      i = save;
      /* fall through to expression */
    } else if (j < n && s[j] == '.') {
      /* p.field = v — load, set field, store back */
      int sl = slot_of(vnames, id.name);
      free(id.name);
      j++;
      Ident fld = parse_ident(s, j, n);
      j = skip(s, fld.i, n);
      if (j < n && s[j] == '=' && !(j + 1 < n && s[j + 1] == '=')) {
        emit2(code, OP_LOAD, sl);
        j = emit_cmp(s, j + 1, n, code, vnames, fnames, faddrs, fargc);
        emit2(code, OP_SET_FIELD, intern_str(fld.name));
        free(fld.name);
        emit2(code, OP_STORE, sl);
        i64a_push(code, OP_POP);
        j = skip(s, j, n);
        if (j < n && s[j] == ';') j++;
        return j;
      }
      free(fld.name);
      i = save;
    } else if (j < n && s[j] == '=' && !(j + 1 < n && s[j + 1] == '=')) {
      j = emit_cmp(s, j + 1, n, code, vnames, fnames, faddrs, fargc);
      int sl = slot_of(vnames, id.name);
      free(id.name);
      emit2(code, OP_STORE, sl);
      i64a_push(code, OP_POP);
      j = skip(s, j, n);
      if (j < n && s[j] == ';') j++;
      return j;
    } else {
      free(id.name);
      i = save;
    }
  }
  i = emit_cmp(s, i, n, code, vnames, fnames, faddrs, fargc);
  i = skip(s, i, n);
  if (i < n && s[i] == ';') {
    i++;
    i64a_push(code, OP_POP); /* statement form; bare expr is implicit return */
  }
  return i;
}

static Program compile_lite(const char *src) {
  size_t n = strlen(src);
  stypes_reset();
  pl_mems_reset();
  tl_tensors_reset();
  ag_clear();
  g_loop_sp = -1;
  g_tmpn = 0;
  call_patches_reset();
  StrA fnames = {0};
  StrA fbodies = {0};
  I64AA fparams = {0};
  EvBind *binds = NULL;
  size_t nbinds = 0, binds_cap = 0;
  int on_serial = 0;
  size_t i = 0;
  while (1) {
    i = skip(src, i, n);
    if (i >= n) break;
    if (starts_kw(src, i, n, "struct")) {
      i = skip(src, i + 6, n);
      Ident id = parse_ident(src, i, n);
      if (find_struct_type(id.name) >= 0) {
        free(id.name);
        die("duplicate struct");
      }
      i = skip(src, id.i, n);
      if (i >= n || src[i] != '{') {
        free(id.name);
        die("expected { after struct name");
      }
      i++;
      StructType st;
      st.name = id.name;
      memset(&st.fields, 0, sizeof(st.fields));
      i = skip(src, i, n);
      if (i < n && src[i] != '}') {
        for (;;) {
          Ident fn = parse_ident(src, i, n);
          if (field_index_of(&st, fn.name) >= 0) {
            free(fn.name);
            free(st.name);
            stra_free(&st.fields);
            die("duplicate struct field");
          }
          stra_push(&st.fields, fn.name);
          i = skip_type_annot(src, fn.i, n);
          i = skip(src, i, n);
          if (i < n && src[i] == ',') {
            i++;
            i = skip(src, i, n);
            continue;
          }
          break;
        }
      }
      i = skip(src, i, n);
      if (i >= n || src[i] != '}') {
        free(st.name);
        stra_free(&st.fields);
        die("expected } after struct fields");
      }
      i++;
      if (g_nstypes + 1 > g_stypes_cap) {
        g_stypes_cap = g_stypes_cap ? g_stypes_cap * 2 : 4;
        g_stypes = (StructType *)xrealloc(g_stypes, g_stypes_cap * sizeof(StructType));
      }
      g_stypes[g_nstypes++] = st;
      continue;
    }
    if (starts_kw(src, i, n, "on")) {
      char *evname;
      char fname[64];
      size_t start;
      int depth;
      int arity = 0;
      StrA pnames = {0};
      i = skip(src, i + 2, n);
      if (i >= n || src[i] != '"') die("expected event string after on");
      i++;
      {
        size_t a = i;
        while (i < n && src[i] != '"') {
          if (src[i] == '\\' && i + 1 < n) i += 2;
          else i++;
        }
        if (i >= n) die("unclosed event string");
        evname = slice_dup(src, a, i);
        i++;
      }
      i = skip(src, i, n);
      if (i >= n || src[i] != '(') {
        free(evname);
        die("expected ( after on event");
      }
      i++;
      i = skip(src, i, n);
      if (i < n && src[i] != ')') {
        for (;;) {
          Ident pn = parse_ident(src, i, n);
          stra_push(&pnames, pn.name);
          i = skip_type_annot(src, pn.i, n);
          i = skip(src, i, n);
          if (i < n && src[i] == ',') {
            i++;
            continue;
          }
          break;
        }
      }
      if (i >= n || src[i] != ')') {
        free(evname);
        die("expected ) in on");
      }
      i = skip(src, i + 1, n);
      if (i >= n || src[i] != '{') {
        free(evname);
        die("expected { on body");
      }
      depth = 0;
      start = i;
      i = match_brace_block(src, i, n);
      (void)depth;
      arity = (int)pnames.len;
      snprintf(fname, sizeof fname, "__on_%d", on_serial++);
      stra_push(&fnames, xstrdup(fname));
      stra_push(&fbodies, slice_dup(src, start, i));
      {
        I64A argc_holder = {0};
        i64a_push(&argc_holder, (int64_t)arity);
        i64aa_push(&fparams, argc_holder);
      }
      {
        size_t last = fbodies.len - 1;
        size_t need = 1, pi;
        char *hdr, *p;
        for (pi = 0; pi < pnames.len; pi++) need += strlen(pnames.data[pi]) + 1;
        hdr = (char *)malloc(need + strlen(fbodies.data[last]) + 1);
        if (!hdr) die("oom");
        p = hdr;
        for (pi = 0; pi < pnames.len; pi++) {
          size_t L = strlen(pnames.data[pi]);
          memcpy(p, pnames.data[pi], L);
          p += L;
          *p++ = (pi + 1 < pnames.len) ? ',' : '|';
        }
        if (pnames.len == 0) *p++ = '|';
        strcpy(p, fbodies.data[last]);
        free(fbodies.data[last]);
        fbodies.data[last] = hdr;
        for (pi = 0; pi < pnames.len; pi++) free(pnames.data[pi]);
        free(pnames.data);
      }
      if (nbinds + 1 > binds_cap) {
        binds_cap = binds_cap ? binds_cap * 2 : 4;
        binds = (EvBind *)xrealloc(binds, binds_cap * sizeof(EvBind));
      }
      binds[nbinds].event = evname;
      binds[nbinds].fn_index = (int64_t)(fnames.len - 1);
      binds[nbinds].arity = arity;
      nbinds++;
      continue;
    }
    if (!starts_kw(src, i, n, "fn")) die("expected fn, on, or struct");
    i = skip(src, i + 2, n);
    Ident id = parse_ident(src, i, n);
    i = skip(src, id.i, n);
    if (i >= n || src[i] != '(') die("expected (");
    i++;
    I64A params = {0};
    StrA pnames = {0};
    i = skip(src, i, n);
    if (i < n && src[i] != ')') {
      for (;;) {
        Ident pn = parse_ident(src, i, n);
        stra_push(&pnames, pn.name);
        i = skip_type_annot(src, pn.i, n);
        i = skip(src, i, n);
        if (i < n && src[i] == ',') {
          i++;
          continue;
        }
        break;
      }
    }
    if (i >= n || src[i] != ')') die("expected )");
    i = skip(src, i + 1, n);
    if (starts_with(src, i, n, "->")) {
      i = skip(src, i + 2, n);
      Ident ty = parse_ident(src, i, n);
      free(ty.name);
      i = ty.i;
    }
    i = skip(src, i, n);
    if (i >= n || src[i] != '{') die("expected { body");
    size_t start = i;
    i = match_brace_block(src, i, n);
    stra_push(&fnames, id.name);
    stra_push(&fbodies, slice_dup(src, start, i));
    I64A argc_holder = {0};
    i64a_push(&argc_holder, (int64_t)pnames.len);
    i64aa_push(&fparams, argc_holder);
    (void)params;
    {
      size_t last = fbodies.len - 1;
      size_t need = 1;
      for (size_t pi = 0; pi < pnames.len; pi++) need += strlen(pnames.data[pi]) + 1;
      char *hdr = (char *)malloc(need + strlen(fbodies.data[last]) + 1);
      if (!hdr) die("oom");
      char *p = hdr;
      for (size_t pi = 0; pi < pnames.len; pi++) {
        size_t L = strlen(pnames.data[pi]);
        memcpy(p, pnames.data[pi], L);
        p += L;
        *p++ = (pi + 1 < pnames.len) ? ',' : '|';
      }
      if (pnames.len == 0) *p++ = '|';
      strcpy(p, fbodies.data[last]);
      free(fbodies.data[last]);
      fbodies.data[last] = hdr;
      for (size_t pi = 0; pi < pnames.len; pi++) free(pnames.data[pi]);
      free(pnames.data);
    }
  }

  I64A fargc = {0};
  for (size_t fi = 0; fi < fnames.len; fi++) {
    i64a_push(&fargc, fparams.data[fi].data[0]);
  }

  Program prog = {0};
  g_strtab = &prog.strs;
  prog.stypes = g_stypes;
  prog.nstypes = g_nstypes;
  /* g_stypes stays aliased for emit_factor struct literals until cleanup */
  I64A *code = &prog.code;
  I64A faddrs = {0};
  emit2(code, OP_JMP, 0);

  for (size_t fi = 0; fi < fnames.len; fi++) {
    i64a_push(&faddrs, (int64_t)code->len);
    StrA vnames = {0};
    char *enc = fbodies.data[fi];
    char *bar = strchr(enc, '|');
    if (!bar) die("internal: missing |");
    if (bar != enc) {
      char *p = enc;
      while (p < bar) {
        char *comma = memchr(p, ',', (size_t)(bar - p));
        size_t len = comma ? (size_t)(comma - p) : (size_t)(bar - p);
        char *nm = (char *)malloc(len + 1);
        if (!nm) die("oom");
        memcpy(nm, p, len);
        nm[len] = 0;
        stra_push(&vnames, nm);
        if (!comma) break;
        p = comma + 1;
      }
    }
    const char *body = bar + 1;
    emit_block(body, 0, strlen(body), code, &vnames, &fnames, &faddrs, &fargc);
    i64a_push(code, OP_RET);
    for (size_t vi = 0; vi < vnames.len; vi++) free(vnames.data[vi]);
    free(vnames.data);
  }
  code->data[1] = (int64_t)code->len;
  call_patches_apply(code, &faddrs);
  call_patches_reset();

  {
    size_t bi, fi;
    for (bi = 0; bi < nbinds; bi++) {
      size_t fii = (size_t)binds[bi].fn_index;
      if (fii >= faddrs.len) die("bad event handler index");
      binds[bi].fn_index = faddrs.data[fii]; /* now address */
    }
    prog.binds = binds;
    prog.nbinds = nbinds;
    prog.nfns = fnames.len;
    prog.fns = (FnInfo *)malloc(prog.nfns * sizeof(FnInfo));
    if (prog.nfns && !prog.fns) die("oom");
    for (fi = 0; fi < fnames.len; fi++) {
      prog.fns[fi].name = xstrdup(fnames.data[fi]);
      prog.fns[fi].addr = faddrs.data[fi];
      prog.fns[fi].arity = (int)fargc.data[fi];
    }
  }

  int found = 0;
  int64_t maddr = 0;
  for (size_t mi = 0; mi < fnames.len; mi++) {
    if (strcmp(fnames.data[mi], "main") == 0) {
      found = 1;
      maddr = faddrs.data[mi];
      if (fargc.data[mi] != 0) die("main must take 0 args");
    }
  }
  if (!found) die("no main()");
  emit3(code, OP_CALL, maddr, 0);
  i64a_push(code, OP_HALT);

  for (size_t fi = 0; fi < fnames.len; fi++) {
    free(fnames.data[fi]);
    free(fbodies.data[fi]);
    i64a_free(&fparams.data[fi]);
  }
  free(fnames.data);
  free(fbodies.data);
  free(fparams.data);
  i64a_free(&fargc);
  i64a_free(&faddrs);
  g_strtab = NULL;
  g_stypes = NULL;
  g_nstypes = g_stypes_cap = 0;
  return prog;
}

static void ensure_slot(ValA *slots, int64_t si) {
  while ((int64_t)slots->len <= si) vala_push(slots, V_i64(0));
}

static int64_t as_i64(Value v, const char *ctx) {
  if (v.tag != TAG_I64) die(ctx);
  return v.payload;
}

static void print_val_raw(Value v, StrA *strs, ListHeap *lists, StructHeap *structs,
                          StructType *stypes, size_t nstypes);

static void print_value(Value v, StrA *strs, ListHeap *lists, StructHeap *structs,
                        StructType *stypes, size_t nstypes) {
  print_val_raw(v, strs, lists, structs, stypes, nstypes);
  putchar('\n');
}

static void print_val_raw(Value v, StrA *strs, ListHeap *lists, StructHeap *structs,
                          StructType *stypes, size_t nstypes) {
  if (v.tag == TAG_I64) {
    printf("%lld", (long long)v.payload);
  } else if (v.tag == TAG_F64) {
    printf("%g", f64_bits(v));
  } else if (v.tag == TAG_STR) {
    if (v.payload < 0 || (size_t)v.payload >= strs->len) die("bad str handle");
    printf("%s", strs->data[v.payload]);
  } else if (v.tag == TAG_LIST) {
    if (v.payload < 0 || (size_t)v.payload >= lists->len) die("bad list handle");
    ValA *L = &lists->data[v.payload];
    putchar('[');
    for (size_t i = 0; i < L->len; i++) {
      if (i) printf(", ");
      print_val_raw(L->data[i], strs, lists, structs, stypes, nstypes);
    }
    putchar(']');
  } else if (v.tag == TAG_STRUCT) {
    if (v.payload < 0 || (size_t)v.payload >= structs->len) die("bad struct handle");
    StructObj *o = &structs->data[v.payload];
    if (o->type_id < 0 || (size_t)o->type_id >= nstypes) die("bad struct type");
    printf("%s{", stypes[o->type_id].name);
    for (size_t i = 0; i < o->fields.len; i++) {
      if (i) printf(", ");
      printf("%lld", (long long)o->fields.data[i]);
    }
    putchar('}');
  } else if (v.tag == TAG_MEMORY) {
    ProphetMem *m = pl_get(v.payload);
    printf("Memory(ep=%d, core=%d, steps=%llu, dim=%dx%d, thr=%.2f)", m->nep, m->ncore,
           (unsigned long long)m->model.steps, m->model.dim, m->model.hidden, m->threshold);
  } else if (v.tag == TAG_TENSOR) {
    tl_print(tl_get(v.payload));
  } else {
    die("bad value tag");
  }
}

static int values_eq(Value a, Value b, StrA *strs) {
  if (is_num(a) && is_num(b)) return to_f64(a, "eq") == to_f64(b, "eq");
  if (a.tag != b.tag) return 0;
  if (a.tag == TAG_I64) return a.payload == b.payload;
  if (a.tag == TAG_STR) {
    if (a.payload < 0 || b.payload < 0 || (size_t)a.payload >= strs->len ||
        (size_t)b.payload >= strs->len)
      die("bad str handle");
    return strcmp(strs->data[a.payload], strs->data[b.payload]) == 0;
  }
  if (a.tag == TAG_LIST) return a.payload == b.payload; /* same handle */
  return 0;
}

static int64_t vm_exec(Program *prog) {
  ValA stack = {0};
  ValA slots = {0};
  I64A ret_ips = {0};
  I64A slot_bases = {0};
  I64A stack_bases = {0};
  ListHeap lists = {0};
  StructHeap structs = {0};
  size_t ip = 0;
  int64_t slot_base = 0;
  int64_t pump_left = 0, pump_done = 0;
  size_t pump_ret_ip = 0;
  I64A *code = &prog->code;
  StrA *strs = &prog->strs;
  size_t bi;
  g_strtab = strs;
  ev_reset();
  for (bi = 0; bi < prog->nbinds; bi++)
    ev_listen(prog->binds[bi].event, prog->binds[bi].fn_index, prog->binds[bi].arity);

  while (ip < code->len) {
    int64_t op = code->data[ip++];
    if (op == OP_HALT) {
      int64_t r = 0;
      if (stack.len) {
        Value top = stack.data[stack.len - 1];
        if (top.tag != TAG_I64) die("main must return i64");
        r = top.payload;
      }
      vala_free(&stack);
      vala_free(&slots);
      i64a_free(&ret_ips);
      i64a_free(&slot_bases);
      i64a_free(&stack_bases);
      listh_free(&lists);
      structh_free(&structs);
      pl_mems_reset();
      tl_tensors_reset();
      ag_clear();
      ev_reset();
      return r;
    }
    if (op == OP_CONST) {
      vala_push(&stack, V_i64(code->data[ip++]));
      continue;
    }
    if (op == OP_CONST_F64) {
      int64_t bits = code->data[ip++];
      double d;
      memcpy(&d, &bits, sizeof(double));
      vala_push(&stack, V_f64(d));
      continue;
    }
    if (op == OP_CONST_STR) {
      vala_push(&stack, V_str(code->data[ip++]));
      continue;
    }
    if (op == OP_LOAD) {
      int64_t si = slot_base + code->data[ip++];
      ensure_slot(&slots, si);
      vala_push(&stack, slots.data[si]);
      continue;
    }
    if (op == OP_STORE) {
      int64_t si = slot_base + code->data[ip++];
      ensure_slot(&slots, si);
      if (!stack.len) die("stack underflow STORE");
      Value v = stack.data[--stack.len];
      slots.data[si] = v;
      vala_push(&stack, v);
      continue;
    }
    if (op == OP_PRINTLN) {
      if (!stack.len) die("stack underflow PRINTLN");
      Value v = stack.data[--stack.len];
      print_value(v, strs, &lists, &structs, prog->stypes, prog->nstypes);
      continue;
    }
    if (op == OP_LIST_NEW) {
      ValA empty = {0};
      int64_t h = (int64_t)lists.len;
      listh_push(&lists, empty);
      vala_push(&stack, V_list(h));
      continue;
    }
    if (op == OP_LIST_PUSH) {
      if (stack.len < 2) die("stack underflow LIST_PUSH");
      Value vv = stack.data[--stack.len];
      Value lv = stack.data[--stack.len];
      if (lv.tag != TAG_LIST) die("push: not a list");
      if (lv.payload < 0 || (size_t)lv.payload >= lists.len) die("bad list handle");
      vala_push(&lists.data[lv.payload], vv);
      vala_push(&stack, lv);
      continue;
    }
    if (op == OP_LEN) {
      if (!stack.len) die("stack underflow LEN");
      Value lv = stack.data[--stack.len];
      if (lv.tag == TAG_LIST) {
        if (lv.payload < 0 || (size_t)lv.payload >= lists.len) die("bad list handle");
        vala_push(&stack, V_i64((int64_t)lists.data[lv.payload].len));
      } else if (lv.tag == TAG_STR) {
        if (lv.payload < 0 || (size_t)lv.payload >= strs->len) die("bad str handle");
        vala_push(&stack, V_i64((int64_t)strlen(strs->data[lv.payload])));
      } else {
        die("len: not a list/str");
      }
      continue;
    }
    if (op == OP_GET) {
      if (stack.len < 2) die("stack underflow GET");
      Value iv = stack.data[--stack.len];
      Value lv = stack.data[--stack.len];
      int64_t idx = as_i64(iv, "index must be i64");
      if (lv.tag == TAG_LIST) {
        if (lv.payload < 0 || (size_t)lv.payload >= lists.len) die("bad list handle");
        ValA *L = &lists.data[lv.payload];
        if (idx < 0 || (size_t)idx >= L->len) die("index out of bounds");
        vala_push(&stack, L->data[idx]);
      } else if (lv.tag == TAG_STR) {
        if (lv.payload < 0 || (size_t)lv.payload >= strs->len) die("bad str handle");
        const char *s = strs->data[lv.payload];
        size_t n = strlen(s);
        if (idx < 0 || (size_t)idx >= n) die("str index out of bounds");
        char buf[2] = {s[idx], 0};
        int h = intern_str(buf);
        vala_push(&stack, V_str(h));
      } else {
        die("index: not a list/str");
      }
      continue;
    }
    if (op == OP_SET) {
      if (stack.len < 3) die("stack underflow SET");
      Value vv = stack.data[--stack.len];
      Value iv = stack.data[--stack.len];
      Value lv = stack.data[--stack.len];
      int64_t idx = as_i64(iv, "index must be i64");
      if (lv.tag != TAG_LIST) die("index assign: not a list");
      if (lv.payload < 0 || (size_t)lv.payload >= lists.len) die("bad list handle");
      ValA *L = &lists.data[lv.payload];
      if (idx < 0 || (size_t)idx >= L->len) die("index out of bounds");
      L->data[idx] = vv;
      vala_push(&stack, vv);
      continue;
    }
    if (op == OP_ORD) {
      if (!stack.len) die("stack underflow ORD");
      Value v = stack.data[--stack.len];
      if (v.tag != TAG_STR) die("ord expects str");
      if (v.payload < 0 || (size_t)v.payload >= strs->len) die("bad str handle");
      const char *s = strs->data[v.payload];
      vala_push(&stack, V_i64(s[0] ? (unsigned char)s[0] : 0));
      continue;
    }
    if (op == OP_MEM_CONFIG) {
      if (stack.len < 3) die("stack underflow memory_config");
      Value vc = stack.data[--stack.len];
      Value ve = stack.data[--stack.len];
      Value vt = stack.data[--stack.len];
      int64_t core_cap = as_i64(vc, "memory_config");
      int64_t ep_cap = as_i64(ve, "memory_config");
      double thr = to_f64(vt, "memory_config");
      if (vt.tag == TAG_I64) thr = (double)vt.payload / 100.0;
      vala_push(&stack, V_mem(pl_mem_new(thr, (int)ep_cap, (int)core_cap)));
      continue;
    }
    if (op == OP_MEM_REMEMBER) {
      double pat[PL_DIM_MAX];
      int pn = 0;
      double surprise;
      Value vs, vo, vm;
      ProphetMem *m;
      if (stack.len < 3) die("stack underflow remember");
      vs = stack.data[--stack.len];
      vo = stack.data[--stack.len];
      vm = stack.data[--stack.len];
      if (vm.tag != TAG_MEMORY) die("remember expects Memory");
      surprise = to_f64(vs, "remember surprise");
      if (vs.tag == TAG_I64) surprise = (double)vs.payload / 100.0;
      pl_value_to_pat(vo, &lists, pat, &pn);
      m = pl_get(vm.payload);
      vala_push(&stack, V_i64(pl_remember(m, pat, pn, surprise) ? 1 : 0));
      continue;
    }
    if (op == OP_MEM_SURPRISE) {
      double a[PL_DIM_MAX], b[PL_DIM_MAX];
      int an = 0, bn = 0;
      Value vb, va;
      if (stack.len < 2) die("stack underflow surprise");
      vb = stack.data[--stack.len];
      va = stack.data[--stack.len];
      pl_value_to_pat(va, &lists, a, &an);
      pl_value_to_pat(vb, &lists, b, &bn);
      vala_push(&stack, V_f64(pl_surprise(a, an, b, bn)));
      continue;
    }
    if (op == OP_MEM_FORESEE) {
      double pat[PL_DIM_MAX], out[PL_DIM_MAX];
      int pn = 0, on = 0;
      Value vo, vm;
      if (stack.len < 2) die("stack underflow foresee");
      vo = stack.data[--stack.len];
      vm = stack.data[--stack.len];
      if (vm.tag != TAG_MEMORY) die("foresee expects Memory");
      pl_value_to_pat(vo, &lists, pat, &pn);
      pl_foresee(pl_get(vm.payload), pat, pn, out, &on);
      vala_push(&stack, pl_pat_to_list(out, on, &lists));
      continue;
    }
    if (op == OP_MEM_CONSOLIDATE) {
      Value vm;
      if (!stack.len) die("stack underflow consolidate");
      vm = stack.data[--stack.len];
      if (vm.tag != TAG_MEMORY) die("consolidate expects Memory");
      vala_push(&stack, V_i64(pl_consolidate(pl_get(vm.payload))));
      continue;
    }
    if (op == OP_MEM_STATS) {
      Value vm;
      if (!stack.len) die("stack underflow mem_stats");
      vm = stack.data[--stack.len];
      if (vm.tag != TAG_MEMORY) die("mem_stats expects Memory");
      vala_push(&stack, pl_stats(pl_get(vm.payload), &lists));
      continue;
    }
    if (op == OP_MEM_RECALL) {
      double pat[PL_DIM_MAX];
      int pn = 0;
      Value vk, vq, vm;
      int64_t k;
      if (stack.len < 3) die("stack underflow recall");
      vk = stack.data[--stack.len];
      vq = stack.data[--stack.len];
      vm = stack.data[--stack.len];
      if (vm.tag != TAG_MEMORY) die("recall expects Memory");
      k = as_i64(vk, "recall k");
      pl_value_to_pat(vq, &lists, pat, &pn);
      vala_push(&stack, pl_recall(pl_get(vm.payload), pat, pn, (int)k, &lists));
      continue;
    }
    if (op == OP_SAVE_MIND) {
      Value vp, vm;
      const char *path;
      if (stack.len < 2) die("stack underflow save_mind");
      vp = stack.data[--stack.len];
      vm = stack.data[--stack.len];
      if (vm.tag != TAG_MEMORY) die("save_mind expects Memory");
      if (vp.tag != TAG_STR) die("save_mind path must be str");
      if (vp.payload < 0 || (size_t)vp.payload >= strs->len) die("bad str");
      path = strs->data[vp.payload];
      if (!pl_save_mind(pl_get(vm.payload), path)) die("save_mind failed");
      vala_push(&stack, V_i64(1));
      continue;
    }
    if (op == OP_LOAD_MIND) {
      Value vp;
      const char *path;
      if (!stack.len) die("stack underflow load_mind");
      vp = stack.data[--stack.len];
      if (vp.tag != TAG_STR) die("load_mind path must be str");
      if (vp.payload < 0 || (size_t)vp.payload >= strs->len) die("bad str");
      path = strs->data[vp.payload];
      vala_push(&stack, V_mem(pl_load_mind(path)));
      continue;
    }
    if (op == OP_TENSOR) {
      int argc = (int)code->data[ip++];
      int shape[TL_RANK_MAX];
      int i;
      if (argc < 1 || argc > TL_RANK_MAX) die("bad tensor argc");
      if ((int)stack.len < argc) die("stack underflow tensor");
      for (i = 0; i < argc; i++) {
        Value v = stack.data[--stack.len];
        shape[i] = (int)as_i64(v, "tensor dim");
      }
      for (i = 0; i < argc / 2; i++) {
        int tmp = shape[i];
        shape[i] = shape[argc - 1 - i];
        shape[argc - 1 - i] = tmp;
      }
      vala_push(&stack, V_tensor(tl_zeros(shape, argc)));
      continue;
    }
    if (op == OP_T_FROM) {
      Value vd, vs;
      int shape[TL_RANK_MAX], rank = 0, n;
      double *buf;
      if (stack.len < 2) die("stack underflow t_from");
      vd = stack.data[--stack.len];
      vs = stack.data[--stack.len];
      tl_shape_from_list(vs, &lists, shape, &rank);
      n = tl_product(shape, rank);
      buf = n > 0 ? (double *)malloc((size_t)n * sizeof(double)) : NULL;
      if (n > 0 && !buf) die("oom");
      if (n > 0) tl_data_from_list(vd, &lists, buf, n);
      {
        int64_t h = tl_alloc(shape, rank, 0);
        if (n > 0) {
          memcpy(tl_get(h)->data, buf, (size_t)n * sizeof(double));
          free(buf);
        }
        vala_push(&stack, V_tensor(h));
      }
      continue;
    }
    if (op == OP_T_FILL) {
      Value vv, vt;
      double x;
      Tensor *t;
      int i;
      if (stack.len < 2) die("stack underflow t_fill");
      vv = stack.data[--stack.len];
      vt = stack.data[--stack.len];
      if (vt.tag != TAG_TENSOR) die("t_fill expects Tensor");
      x = to_f64(vv, "t_fill");
      t = tl_get(vt.payload);
      for (i = 0; i < t->n; i++) t->data[i] = x;
      vala_push(&stack, vt);
      continue;
    }
    if (op == OP_T_GET) {
      Value vi, vt;
      int64_t idx;
      Tensor *t;
      if (stack.len < 2) die("stack underflow t_get");
      vi = stack.data[--stack.len];
      vt = stack.data[--stack.len];
      if (vt.tag != TAG_TENSOR) die("t_get expects Tensor");
      idx = as_i64(vi, "t_get index");
      t = tl_get(vt.payload);
      if (idx < 0 || idx >= t->n) die("t_get out of range");
      vala_push(&stack, V_f64(t->data[idx]));
      continue;
    }
    if (op == OP_T_SET) {
      Value vv, vi, vt;
      int64_t idx;
      Tensor *t;
      if (stack.len < 3) die("stack underflow t_set");
      vv = stack.data[--stack.len];
      vi = stack.data[--stack.len];
      vt = stack.data[--stack.len];
      if (vt.tag != TAG_TENSOR) die("t_set expects Tensor");
      idx = as_i64(vi, "t_set index");
      t = tl_get(vt.payload);
      if (idx < 0 || idx >= t->n) die("t_set out of range");
      t->data[idx] = to_f64(vv, "t_set");
      vala_push(&stack, vt);
      continue;
    }
    if (op == OP_T_SHAPE) {
      Value vt;
      if (!stack.len) die("stack underflow t_shape");
      vt = stack.data[--stack.len];
      if (vt.tag != TAG_TENSOR) die("t_shape expects Tensor");
      vala_push(&stack, tl_shape_to_list(tl_get(vt.payload), &lists));
      continue;
    }
    if (op == OP_T_ADD || op == OP_T_SUB || op == OP_T_MUL || op == OP_T_MATMUL ||
        op == OP_T_DOT) {
      Value vb, va;
      if (stack.len < 2) die("stack underflow tensor binary");
      vb = stack.data[--stack.len];
      va = stack.data[--stack.len];
      if (va.tag != TAG_TENSOR || vb.tag != TAG_TENSOR) die("tensor op expects Tensor");
      if (op == OP_T_ADD)
        vala_push(&stack, V_tensor(tl_ew(va.payload, vb.payload, tl_op_add)));
      else if (op == OP_T_SUB)
        vala_push(&stack, V_tensor(tl_ew(va.payload, vb.payload, tl_op_sub)));
      else if (op == OP_T_MUL)
        vala_push(&stack, V_tensor(tl_ew(va.payload, vb.payload, tl_op_mul)));
      else if (op == OP_T_MATMUL)
        vala_push(&stack, V_tensor(tl_matmul(va.payload, vb.payload)));
      else
        vala_push(&stack, V_f64(tl_dot(va.payload, vb.payload)));
      continue;
    }
    if (op == OP_T_RESHAPE) {
      Value vs, vt;
      int shape[TL_RANK_MAX], rank = 0, n;
      Tensor *t;
      if (stack.len < 2) die("stack underflow t_reshape");
      vs = stack.data[--stack.len];
      vt = stack.data[--stack.len];
      if (vt.tag != TAG_TENSOR) die("t_reshape expects Tensor");
      t = tl_get(vt.payload);
      tl_shape_from_list(vs, &lists, shape, &rank);
      n = tl_product(shape, rank);
      if (n != t->n) die("t_reshape size mismatch");
      {
        int64_t h = tl_clone_shape_data(shape, rank, t->data, t->n);
        vala_push(&stack, V_tensor(h));
      }
      continue;
    }
    if (op == OP_T_SCALE) {
      Value vs, vt;
      if (stack.len < 2) die("stack underflow t_scale");
      vs = stack.data[--stack.len];
      vt = stack.data[--stack.len];
      if (vt.tag != TAG_TENSOR) die("t_scale expects Tensor");
      vala_push(&stack, V_tensor(tl_scale(vt.payload, to_f64(vs, "t_scale"))));
      continue;
    }
    if (op == OP_T_TRANSPOSE || op == OP_T_EXP || op == OP_T_SOFTMAX || op == OP_T_SUM ||
        op == OP_T_LOG) {
      Value vt;
      if (!stack.len) die("stack underflow tensor unary");
      vt = stack.data[--stack.len];
      if (vt.tag != TAG_TENSOR) die("tensor unary expects Tensor");
      if (op == OP_T_TRANSPOSE)
        vala_push(&stack, V_tensor(tl_transpose(vt.payload)));
      else if (op == OP_T_EXP)
        vala_push(&stack, V_tensor(tl_exp(vt.payload)));
      else if (op == OP_T_SOFTMAX)
        vala_push(&stack, V_tensor(tl_softmax(vt.payload)));
      else if (op == OP_T_LOG)
        vala_push(&stack, V_tensor(tl_log(vt.payload)));
      else
        vala_push(&stack, V_f64(tl_sum(vt.payload)));
      continue;
    }
    if (op == OP_SWEEP) {
      vala_push(&stack, V_i64(0));
      continue;
    }
    if (op == OP_NOW_MS) {
      vala_push(&stack, V_i64(0));
      continue;
    }
    if (op == OP_LOAD_PPM || op == OP_LOAD_WAV || op == OP_LOAD_TENSOR || op == OP_READ_FILE) {
      Value vp;
      const char *path;
      if (!stack.len) die("stack underflow load path");
      vp = stack.data[--stack.len];
      if (vp.tag != TAG_STR) die("path must be str");
      if (vp.payload < 0 || (size_t)vp.payload >= strs->len) die("bad str");
      path = strs->data[vp.payload];
      if (op == OP_LOAD_PPM)
        vala_push(&stack, V_tensor(tl_load_ppm(path)));
      else if (op == OP_LOAD_WAV)
        vala_push(&stack, V_tensor(tl_load_wav(path)));
      else if (op == OP_LOAD_TENSOR)
        vala_push(&stack, V_tensor(tl_load_tensor(path)));
      else {
        FILE *f = fopen(path, "rb");
        long sz;
        char *buf;
        size_t nread;
        if (!f) die("read_file: cannot open");
        if (fseek(f, 0, SEEK_END) != 0) {
          fclose(f);
          die("read_file: seek");
        }
        sz = ftell(f);
        if (sz < 0) {
          fclose(f);
          die("read_file: tell");
        }
        rewind(f);
        buf = (char *)malloc((size_t)sz + 1);
        if (!buf) {
          fclose(f);
          die("oom");
        }
        nread = fread(buf, 1, (size_t)sz, f);
        fclose(f);
        buf[nread] = 0;
        vala_push(&stack, V_str(intern_str(buf)));
        free(buf);
      }
      continue;
    }
    if (op == OP_SAVE_TENSOR) {
      Value vp, vt;
      const char *path;
      if (stack.len < 2) die("stack underflow save_tensor");
      vp = stack.data[--stack.len];
      vt = stack.data[--stack.len];
      if (vt.tag != TAG_TENSOR) die("save_tensor expects Tensor");
      if (vp.tag != TAG_STR) die("save_tensor path must be str");
      path = strs->data[vp.payload];
      if (!tl_save_tensor(vt.payload, path)) die("save_tensor failed");
      vala_push(&stack, V_i64(1));
      continue;
    }
    if (op == OP_WRITE_FILE) {
      Value vc, vp;
      const char *path, *body;
      FILE *f;
      if (stack.len < 2) die("stack underflow write_file");
      vc = stack.data[--stack.len];
      vp = stack.data[--stack.len];
      if (vp.tag != TAG_STR || vc.tag != TAG_STR) die("write_file expects str, str");
      path = strs->data[vp.payload];
      body = strs->data[vc.payload];
      f = fopen(path, "wb");
      if (!f) die("write_file: cannot open");
      fwrite(body, 1, strlen(body), f);
      fclose(f);
      vala_push(&stack, V_i64(1));
      continue;
    }
    if (op == OP_T_MEAN) {
      Value vt;
      double f = 0.0;
      int is_f = 0;
      int64_t h;
      if (!stack.len) die("stack underflow t_mean");
      vt = stack.data[--stack.len];
      if (vt.tag != TAG_TENSOR) die("t_mean expects Tensor");
      h = tl_mean_tensor(vt.payload, &f, &is_f);
      if (is_f)
        vala_push(&stack, V_f64(f));
      else
        vala_push(&stack, V_tensor(h));
      continue;
    }
    if (op == OP_T_MSE) {
      Value vb, va;
      if (stack.len < 2) die("stack underflow t_mse");
      vb = stack.data[--stack.len];
      va = stack.data[--stack.len];
      if (va.tag != TAG_TENSOR || vb.tag != TAG_TENSOR) die("t_mse expects Tensor");
      vala_push(&stack, V_f64(tl_mse_ids(va.payload, vb.payload)));
      continue;
    }
    if (op == OP_T_PATCH_MEAN) {
      Value vgw, vgh, vt;
      if (stack.len < 3) die("stack underflow t_patch_mean");
      vgw = stack.data[--stack.len];
      vgh = stack.data[--stack.len];
      vt = stack.data[--stack.len];
      if (vt.tag != TAG_TENSOR) die("t_patch_mean expects Tensor");
      vala_push(&stack, V_tensor(tl_patch_mean(vt.payload, (int)as_i64(vgh, "t_patch_mean"),
                                              (int)as_i64(vgw, "t_patch_mean"))));
      continue;
    }
    if (op == OP_T_LINEAR_GRAD) {
      Value vy, vx, vw;
      if (stack.len < 3) die("stack underflow t_linear_grad");
      vy = stack.data[--stack.len];
      vx = stack.data[--stack.len];
      vw = stack.data[--stack.len];
      if (vw.tag != TAG_TENSOR || vx.tag != TAG_TENSOR || vy.tag != TAG_TENSOR)
        die("t_linear_grad expects Tensor");
      vala_push(&stack, V_tensor(tl_linear_grad(vw.payload, vx.payload, vy.payload)));
      continue;
    }
    if (op == OP_LEARN) {
      double x[PL_DIM_MAX], y[PL_DIM_MAX];
      int xn = 0, yn = 0;
      Value vy, vx, vm;
      if (stack.len < 3) die("stack underflow learn");
      vy = stack.data[--stack.len];
      vx = stack.data[--stack.len];
      vm = stack.data[--stack.len];
      if (vm.tag != TAG_MEMORY) die("learn expects Memory");
      pl_value_to_pat(vx, &lists, x, &xn);
      pl_value_to_pat(vy, &lists, y, &yn);
      vala_push(&stack, V_f64(pl_learn(pl_get(vm.payload), x, xn, y, yn)));
      continue;
    }
    if (op == OP_PREDICT) {
      double pat[PL_DIM_MAX], out[PL_DIM_MAX];
      int pn = 0, on = 0;
      Value vo, vm;
      if (stack.len < 2) die("stack underflow predict");
      vo = stack.data[--stack.len];
      vm = stack.data[--stack.len];
      if (vm.tag != TAG_MEMORY) die("predict expects Memory");
      pl_value_to_pat(vo, &lists, pat, &pn);
      pl_predict(pl_get(vm.payload), pat, pn, out, &on);
      vala_push(&stack, pl_pat_to_list(out, on, &lists));
      continue;
    }
    if (op == OP_UNROLL) {
      double pat[PL_DIM_MAX], traj[64 * PL_DIM_MAX];
      int pn = 0, steps, os = 0, od = 0, s;
      Value vk, vo, vm;
      ValA empty = {0};
      int64_t oh;
      if (stack.len < 3) die("stack underflow unroll");
      vk = stack.data[--stack.len];
      vo = stack.data[--stack.len];
      vm = stack.data[--stack.len];
      if (vm.tag != TAG_MEMORY) die("unroll expects Memory");
      steps = (int)as_i64(vk, "unroll steps");
      pl_value_to_pat(vo, &lists, pat, &pn);
      pl_unroll(pl_get(vm.payload), pat, pn, steps, traj, &os, &od);
      oh = (int64_t)lists.len;
      listh_push(&lists, empty);
      for (s = 0; s < os; s++) {
        Value inner = pl_pat_to_list(traj + s * PL_DIM_MAX, od, &lists);
        vala_push(&lists.data[oh], inner);
      }
      vala_push(&stack, V_list(oh));
      continue;
    }
    if (op == OP_REMEMBER_NEXT) {
      double a[PL_DIM_MAX], b[PL_DIM_MAX];
      int an = 0, bn = 0;
      double surprise;
      Value vs, vn, vo, vm;
      if (stack.len < 4) die("stack underflow remember_next");
      vs = stack.data[--stack.len];
      vn = stack.data[--stack.len];
      vo = stack.data[--stack.len];
      vm = stack.data[--stack.len];
      if (vm.tag != TAG_MEMORY) die("remember_next expects Memory");
      surprise = to_f64(vs, "remember_next surprise");
      if (vs.tag == TAG_I64) surprise = (double)vs.payload / 100.0;
      pl_value_to_pat(vo, &lists, a, &an);
      pl_value_to_pat(vn, &lists, b, &bn);
      vala_push(&stack, V_i64(pl_remember_pair(pl_get(vm.payload), a, an, b, bn, surprise, 0) ? 1 : 0));
      continue;
    }
    if (op == OP_EMIT) {
      Value vv, ve;
      const char *ev;
      if (stack.len < 2) die("stack underflow emit");
      vv = stack.data[--stack.len];
      ve = stack.data[--stack.len];
      if (ve.tag != TAG_STR) die("emit event must be str");
      if (ve.payload < 0 || (size_t)ve.payload >= strs->len) die("bad str");
      ev = strs->data[ve.payload];
      ev_emit(ev, vv);
      vala_push(&stack, V_i64(0));
      continue;
    }
    if (op == OP_PENDING) {
      vala_push(&stack, V_i64(ev_pending()));
      continue;
    }
    if (op == OP_LISTEN) {
      Value vh, ve;
      const char *ev, *hn;
      size_t fi;
      int found = 0;
      if (stack.len < 2) die("stack underflow listen");
      vh = stack.data[--stack.len];
      ve = stack.data[--stack.len];
      if (ve.tag != TAG_STR || vh.tag != TAG_STR) die("listen expects str, str");
      ev = strs->data[ve.payload];
      hn = strs->data[vh.payload];
      for (fi = 0; fi < prog->nfns; fi++) {
        if (strcmp(prog->fns[fi].name, hn) == 0) {
          ev_listen(ev, prog->fns[fi].addr, prog->fns[fi].arity);
          found = 1;
          break;
        }
      }
      if (!found) die("listen: unknown handler");
      vala_push(&stack, V_i64(0));
      continue;
    }
    if (op == OP_TYPEOF) {
      Value v;
      const char *tn = "unknown";
      if (!stack.len) die("stack underflow typeof");
      v = stack.data[--stack.len];
      if (v.tag == TAG_I64) tn = "i64";
      else if (v.tag == TAG_F64) tn = "f64";
      else if (v.tag == TAG_STR) tn = "str";
      else if (v.tag == TAG_LIST) tn = "list";
      else if (v.tag == TAG_STRUCT) tn = "struct";
      else if (v.tag == TAG_MEMORY) tn = "Memory";
      else if (v.tag == TAG_TENSOR) tn = "Tensor";
      vala_push(&stack, V_str(intern_str(tn)));
      continue;
    }
    if (op == OP_AG_CLEAR) {
      ag_clear();
      vala_push(&stack, V_i64(0));
      continue;
    }
    if (op == OP_AG_PARAM) {
      Value vt;
      if (!stack.len) die("stack underflow ag_param");
      vt = stack.data[--stack.len];
      if (vt.tag != TAG_TENSOR) die("ag_param expects Tensor");
      vala_push(&stack, V_i64(ag_param(vt.payload)));
      continue;
    }
    if (op == OP_AG_CONST) {
      Value vt;
      if (!stack.len) die("stack underflow ag_const");
      vt = stack.data[--stack.len];
      if (vt.tag == TAG_TENSOR)
        vala_push(&stack, V_i64(ag_const_t(vt.payload)));
      else if (vt.tag == TAG_F64 || vt.tag == TAG_I64)
        vala_push(&stack, V_i64(ag_const_f(to_f64(vt, "ag_const"))));
      else
        die("ag_const expects Tensor|number");
      continue;
    }
    if (op == OP_AG_ADD || op == OP_AG_SUB || op == OP_AG_MUL || op == OP_AG_MATMUL ||
        op == OP_AG_MSE) {
      Value vb, va;
      int a, b;
      if (stack.len < 2) die("stack underflow ag binary");
      vb = stack.data[--stack.len];
      va = stack.data[--stack.len];
      a = (int)as_i64(va, "ag node");
      b = (int)as_i64(vb, "ag node");
      if (op == OP_AG_ADD)
        vala_push(&stack, V_i64(ag_add_ids(a, b)));
      else if (op == OP_AG_SUB)
        vala_push(&stack, V_i64(ag_sub_ids(a, b)));
      else if (op == OP_AG_MUL)
        vala_push(&stack, V_i64(ag_mul_ids(a, b)));
      else if (op == OP_AG_MATMUL)
        vala_push(&stack, V_i64(ag_matmul_ids(a, b)));
      else
        vala_push(&stack, V_i64(ag_mse_n(a, b)));
      continue;
    }
    if (op == OP_AG_SCALE) {
      Value vs, va;
      if (stack.len < 2) die("stack underflow ag_scale");
      vs = stack.data[--stack.len];
      va = stack.data[--stack.len];
      vala_push(&stack, V_i64(ag_scale_n((int)as_i64(va, "ag_scale"), to_f64(vs, "ag_scale"))));
      continue;
    }
    if (op == OP_AG_RESHAPE) {
      Value vs, va;
      int shape[TL_RANK_MAX], rank = 0;
      if (stack.len < 2) die("stack underflow ag_reshape");
      vs = stack.data[--stack.len];
      va = stack.data[--stack.len];
      tl_shape_from_list(vs, &lists, shape, &rank);
      vala_push(&stack, V_i64(ag_reshape_n((int)as_i64(va, "ag_reshape"), shape, rank)));
      continue;
    }
    if (op == OP_AG_STEP) {
      Value vs, va;
      if (stack.len < 2) die("stack underflow ag_step");
      vs = stack.data[--stack.len];
      va = stack.data[--stack.len];
      vala_push(&stack, V_tensor(ag_step_n((int)as_i64(va, "ag_step"), to_f64(vs, "ag_step"))));
      continue;
    }
    if (op == OP_AG_RELU || op == OP_AG_NEG || op == OP_AG_TRANSPOSE || op == OP_AG_EXP ||
        op == OP_AG_LOG || op == OP_AG_SOFTMAX || op == OP_AG_SUM || op == OP_AG_VALUE ||
        op == OP_AG_GRAD || op == OP_AG_BACKWARD) {
      Value va;
      int id;
      if (!stack.len) die("stack underflow ag unary");
      va = stack.data[--stack.len];
      id = (int)as_i64(va, "ag unary");
      if (op == OP_AG_RELU)
        vala_push(&stack, V_i64(ag_relu_n(id)));
      else if (op == OP_AG_NEG)
        vala_push(&stack, V_i64(ag_neg_n(id)));
      else if (op == OP_AG_TRANSPOSE)
        vala_push(&stack, V_i64(ag_transpose_n(id)));
      else if (op == OP_AG_EXP)
        vala_push(&stack, V_i64(ag_exp_n(id)));
      else if (op == OP_AG_LOG)
        vala_push(&stack, V_i64(ag_log_n(id)));
      else if (op == OP_AG_SOFTMAX)
        vala_push(&stack, V_i64(ag_softmax_n(id)));
      else if (op == OP_AG_SUM)
        vala_push(&stack, V_i64(ag_sum_n(id)));
      else if (op == OP_AG_VALUE)
        vala_push(&stack, ag_value_v(id));
      else if (op == OP_AG_GRAD)
        vala_push(&stack, ag_grad_v(id));
      else {
        ag_backward(id);
        vala_push(&stack, V_i64(0));
      }
      continue;
    }
    if (op == OP_PUMP) {
      Value vm;
      if (!stack.len) die("stack underflow pump");
      vm = stack.data[--stack.len];
      pump_left = as_i64(vm, "pump");
      if (pump_left < 0) die("pump expects non-negative");
      pump_done = 0;
      pump_ret_ip = ip; /* resume here after draining queue */
    pump_step:
      {
        int launched = 0;
        while (pump_left > 0) {
          char *en = NULL;
          Value ev;
          EvHandler *h;
          if (!ev_dequeue(&en, &ev)) break;
          pump_left--;
          h = ev_find(en);
          free(en);
          if (!h) continue;
          i64a_push(&ret_ips, -2); /* resume pump */
          i64a_push(&slot_bases, slot_base);
          i64a_push(&stack_bases, (int64_t)stack.len);
          slot_base = (int64_t)slots.len;
          if (h->arity == 1) {
            ensure_slot(&slots, slot_base);
            slots.data[slot_base] = ev;
          } else if (h->arity != 0) {
            die("event handler arity must be 0 or 1");
          }
          ip = (size_t)h->addr;
          launched = 1;
          break;
        }
        if (launched) continue;
        ip = pump_ret_ip;
        vala_push(&stack, V_i64(pump_done));
        continue;
      }
    }
    if (op == OP_STRUCT_NEW) {
      int64_t tid = code->data[ip++];
      if (tid < 0 || (size_t)tid >= prog->nstypes) die("bad struct type id");
      StructType *st = &prog->stypes[tid];
      size_t nf = st->fields.len;
      if (stack.len < nf) die("stack underflow STRUCT_NEW");
      StructObj obj;
      obj.type_id = (int)tid;
      memset(&obj.fields, 0, sizeof(obj.fields));
      /* fields were pushed in decl order; top is last field */
      int64_t *tmp = NULL;
      if (nf) {
        tmp = (int64_t *)malloc(nf * sizeof(int64_t));
        if (!tmp) die("oom");
      }
      for (size_t fi = nf; fi > 0;) {
        fi--;
        Value fv = stack.data[--stack.len];
        tmp[fi] = as_i64(fv, "struct fields must be i64");
      }
      for (size_t fi = 0; fi < nf; fi++) i64a_push(&obj.fields, tmp[fi]);
      free(tmp);
      int64_t h = (int64_t)structs.len;
      structh_push(&structs, obj);
      vala_push(&stack, V_struct(h));
      continue;
    }
    if (op == OP_GET_FIELD) {
      int64_t name_idx = code->data[ip++];
      if (!stack.len) die("stack underflow GET_FIELD");
      Value sv = stack.data[--stack.len];
      if (sv.tag != TAG_STRUCT) die("field access: not a struct");
      if (sv.payload < 0 || (size_t)sv.payload >= structs.len) die("bad struct handle");
      if (name_idx < 0 || (size_t)name_idx >= strs->len) die("bad field name");
      StructObj *o = &structs.data[sv.payload];
      if (o->type_id < 0 || (size_t)o->type_id >= prog->nstypes) die("bad struct type");
      int fidx = field_index_of(&prog->stypes[o->type_id], strs->data[name_idx]);
      if (fidx < 0) die("unknown field");
      if ((size_t)fidx >= o->fields.len) die("bad field index");
      vala_push(&stack, V_i64(o->fields.data[fidx]));
      continue;
    }
    if (op == OP_SET_FIELD) {
      int64_t name_idx = code->data[ip++];
      if (stack.len < 2) die("stack underflow SET_FIELD");
      Value vv = stack.data[--stack.len];
      Value sv = stack.data[--stack.len];
      int64_t val = as_i64(vv, "struct fields must be i64");
      if (sv.tag != TAG_STRUCT) die("field assign: not a struct");
      if (sv.payload < 0 || (size_t)sv.payload >= structs.len) die("bad struct handle");
      if (name_idx < 0 || (size_t)name_idx >= strs->len) die("bad field name");
      StructObj *o = &structs.data[sv.payload];
      if (o->type_id < 0 || (size_t)o->type_id >= prog->nstypes) die("bad struct type");
      int fidx = field_index_of(&prog->stypes[o->type_id], strs->data[name_idx]);
      if (fidx < 0) die("unknown field");
      if ((size_t)fidx >= o->fields.len) die("bad field index");
      o->fields.data[fidx] = val;
      vala_push(&stack, sv);
      continue;
    }
#define BIN_NUM(OP_I, OP_F, AS_CMP)                                             \
  do {                                                                          \
    if (stack.len < 2) die("stack underflow");                                  \
    Value vb = stack.data[--stack.len];                                         \
    Value va = stack.data[--stack.len];                                         \
    if (!is_num(va) || !is_num(vb)) die("arith expects number");                \
    if (va.tag == TAG_F64 || vb.tag == TAG_F64) {                               \
      double a = to_f64(va, "arith");                                           \
      double b = to_f64(vb, "arith");                                           \
      if (AS_CMP)                                                               \
        vala_push(&stack, V_i64((OP_F) ? 1 : 0));                               \
      else                                                                      \
        vala_push(&stack, V_f64(OP_F));                                         \
    } else {                                                                    \
      int64_t a = va.payload;                                                   \
      int64_t b = vb.payload;                                                   \
      if (AS_CMP)                                                               \
        vala_push(&stack, V_i64((OP_I) ? 1 : 0));                               \
      else                                                                      \
        vala_push(&stack, V_i64(OP_I));                                         \
    }                                                                           \
  } while (0)
    if (op == OP_ADD) {
      if (stack.len < 2) die("stack underflow");
      Value vb = stack.data[--stack.len];
      Value va = stack.data[--stack.len];
      if (va.tag == TAG_STR && vb.tag == TAG_STR) {
        if (va.payload < 0 || vb.payload < 0 || (size_t)va.payload >= strs->len ||
            (size_t)vb.payload >= strs->len)
          die("bad str handle");
        const char *a = strs->data[va.payload];
        const char *b = strs->data[vb.payload];
        size_t na = strlen(a), nb = strlen(b);
        char *cat = (char *)malloc(na + nb + 1);
        if (!cat) die("oom");
        memcpy(cat, a, na);
        memcpy(cat + na, b, nb + 1);
        int h = intern_str(cat);
        free(cat);
        vala_push(&stack, V_str(h));
      } else {
        /* restore for BIN_NUM path */
        vala_push(&stack, va);
        vala_push(&stack, vb);
        BIN_NUM(a + b, a + b, 0);
      }
      continue;
    }
    if (op == OP_SUB) {
      BIN_NUM(a - b, a - b, 0);
      continue;
    }
    if (op == OP_MUL) {
      BIN_NUM(a * b, a * b, 0);
      continue;
    }
    if (op == OP_DIV) {
      BIN_NUM(a / b, a / b, 0);
      continue;
    }
    if (op == OP_LT) {
      BIN_NUM(a < b, a < b, 1);
      continue;
    }
    if (op == OP_GT) {
      BIN_NUM(a > b, a > b, 1);
      continue;
    }
    if (op == OP_LE) {
      BIN_NUM(a <= b, a <= b, 1);
      continue;
    }
    if (op == OP_GE) {
      BIN_NUM(a >= b, a >= b, 1);
      continue;
    }
#undef BIN_NUM
    if (op == OP_ROUND) {
      if (!stack.len) die("stack underflow ROUND");
      Value v = stack.data[--stack.len];
      vala_push(&stack, V_i64(k_round_d(to_f64(v, "round expects number"))));
      continue;
    }
    if (op == OP_ASSERT) {
      if (!stack.len) die("stack underflow ASSERT");
      Value v = stack.data[--stack.len];
      int64_t ok = as_i64(v, "assert expects i64");
      if (!ok) die("assert failed");
      vala_push(&stack, V_i64(0));
      continue;
    }
    if (op == OP_POP) {
      if (!stack.len) die("stack underflow POP");
      stack.len--;
      continue;
    }
    if (op == OP_EQ) {
      if (stack.len < 2) die("stack underflow");
      Value vb = stack.data[--stack.len];
      Value va = stack.data[--stack.len];
      vala_push(&stack, V_i64(values_eq(va, vb, strs) ? 1 : 0));
      continue;
    }
    if (op == OP_NE) {
      if (stack.len < 2) die("stack underflow");
      Value vb = stack.data[--stack.len];
      Value va = stack.data[--stack.len];
      vala_push(&stack, V_i64(values_eq(va, vb, strs) ? 0 : 1));
      continue;
    }
    if (op == OP_JMP) {
      ip = (size_t)code->data[ip];
      continue;
    }
    if (op == OP_JMPF) {
      int64_t t = code->data[ip++];
      if (!stack.len) die("stack underflow JMPF");
      Value c = stack.data[--stack.len];
      int64_t cv = as_i64(c, "condition must be i64");
      if (cv == 0) ip = (size_t)t;
      continue;
    }
    if (op == OP_CALL) {
      int64_t addr = code->data[ip++];
      int64_t argc = code->data[ip++];
      if ((int64_t)stack.len < argc) die("stack underflow CALL");
      Value args_buf[64];
      if (argc > 64) die("too many args");
      for (int64_t ai = argc - 1; ai >= 0; ai--) args_buf[ai] = stack.data[--stack.len];
      i64a_push(&ret_ips, (int64_t)ip);
      i64a_push(&slot_bases, slot_base);
      i64a_push(&stack_bases, (int64_t)stack.len); /* caller's stack depth */
      slot_base = (int64_t)slots.len;
      for (int64_t ai = 0; ai < argc; ai++) {
        ensure_slot(&slots, slot_base + ai);
        slots.data[slot_base + ai] = args_buf[ai];
      }
      ip = (size_t)addr;
      continue;
    }
    if (op == OP_RET) {
      /* void fn / trailing RET after statements: implicit 0 */
      Value retv = stack.len ? stack.data[--stack.len] : V_i64(0);
      int64_t rip;
      slots.len = (size_t)slot_base;
      if (!slot_bases.len || !ret_ips.len || !stack_bases.len) die("ret with empty frame");
      slot_base = slot_bases.data[--slot_bases.len];
      {
        int64_t sb = stack_bases.data[--stack_bases.len];
        if (sb < 0 || (size_t)sb > stack.len) die("bad stack frame");
        stack.len = (size_t)sb; /* drop callee junk */
      }
      rip = ret_ips.data[--ret_ips.len];
      if (rip == -2) {
        pump_done++;
        goto pump_step;
      }
      ip = (size_t)rip;
      vala_push(&stack, retv);
      continue;
    }
    die("bad opcode");
  }
  {
    int64_t r = 0;
    if (stack.len) {
      Value top = stack.data[stack.len - 1];
      if (top.tag != TAG_I64) die("main must return i64");
      r = top.payload;
    }
    vala_free(&stack);
    vala_free(&slots);
    i64a_free(&ret_ips);
    i64a_free(&slot_bases);
    i64a_free(&stack_bases);
    listh_free(&lists);
    structh_free(&structs);
    pl_mems_reset();
    tl_tensors_reset();
    ag_clear();
    ev_reset();
    return r;
  }
}

static int64_t run_lite(const char *src) {
  Program prog = compile_lite(src);
  int64_t r = vm_exec(&prog);
  program_free(&prog);
  return r;
}

#include "generated/rt_host.inc.c"

static int selftest(void) {
  struct {
    const char *src;
    int64_t want;
  } cases[] = {
      {"fn main() { 2 + 3 * 4 }", 14},
      {"fn main() { let x = 2 + 3; let y = x * 4; y }", 20},
      {"fn add(a, b) { a + b } fn main() { add(20, 22) }", 42},
      {"fn main() { let x = 0; while x < 5 { x = x + 1; } x }", 5},
      {"fn main() { let x = 3; if x > 2 { 10 } else { 20 } }", 10},
      {"fn main() -> i64 { let n = 1; let i = 1; while i <= 5 { n = n * i; i = "
       "i + 1; } return n; }",
       120},
      {"fn main() { let xs = [1, 2, 3]; len(xs) }", 3},
      {"fn main() { let xs = []; push(xs, 10); push(xs, 20); xs[0] + xs[1] }", 30},
      {"fn main() { let xs = [10, 20, 30]; let s = 0; let i = 0; while i < "
       "len(xs) { s = s + xs[i]; i = i + 1; } s }",
       60},
      {"fn main() { let xs = [1, 2]; xs[0] = 7; xs[0] + xs[1] }", 9},
      {"fn main() { println(42); 0 }", 0},
      {"fn main() { println(\"hi\"); 0 }", 0},
      {"fn main() { \"ab\" == \"ab\" }", 1},
      {"fn main() { \"ab\" != \"cd\" }", 1},
      {"fn main() { 1 != 2 }", 1},
      {"fn main() { 3 >= 3 }", 1},
      {"fn main() { 2 >= 3 }", 0},
      {"struct Point { x, y } fn main() { let p = Point { x: 3, y: 4 }; p.x }", 3},
      {"struct Point { x, y } fn main() { let p = Point { y: 4, x: 3 }; p.x * p.x + "
       "p.y * p.y }",
       25},
      {"struct Point { x, y } fn main() { let p = Point { x: 3, y: 4 }; p.x = 6; p.x "
       "* p.x + p.y * p.y }",
       52},
      {"struct Point { x, y } fn main() { let p = Point { x: 1, y: 2 }; println(p); "
       "0 }",
       0},
      {"fn main() { let x = 1.5; let y = 2.5; round(x + y) }", 4},
      {"fn main() { if 1.25 * 2.0 > 2.0 { 1 } else { 0 } }", 1},
      {"fn main() { round(2.6) }", 3},
      {"fn main() { let a = 3; let b = 0.5; round(a + b) }", 4},
      {"fn main() { let n = 2; let r = 0; if n == 0 { r = 1; } else if n == 1 { r = 2; } "
       "else if n == 2 { r = 3; } else { r = 4; } r }",
       3},
      {"fn main() { assert(1); 0 }", 0},
      {"fn main() { let x = 1.5; assert(round(x) == 2); 0 }", 0},
      {"fn main() -> i64 { let x: i64 = 21; let y: i64 = 2; x * y }", 42},
      {"fn add(a: i64, b: i64) -> i64 { a + b } fn main() -> i64 { add(20, 22) }",
       42},
      {"fn main() -> i64 { let t: Tensor ttl 5s = tensor(2, 3); sweep(); "
       "round(t_get(t, 0)) }",
       0},
      {"fn main() { let a = t_from([2, 3], [1.0, 2.0, 3.0, 4.0, 5.0, 6.0]); let b = "
       "t_from([3, 2], [1.0, 0.0, 0.0, 1.0, 1.0, 1.0]); let c = t_matmul(a, b); "
       "assert(round(t_get(c, 0)) == 4); assert(round(t_get(c, 3)) == 11); 0 }",
       0},
      {"fn main() { let img = load_ppm(\"../examples/ml/assets/dot.ppm\"); let m = "
       "t_mean(img); assert(len(t_shape(m)) == 1); 0 }",
       0},
      {"on \"ping\"(x) { emit(\"pong\", x + 1); } on \"pong\"(x) { } fn main() { "
       "emit(\"ping\", 1); assert(pump(8) == 2); assert(pending() == 0); 0 }",
       0},
      {"fn main() { let s = 0; for i in 0..5 { s = s + i; } s }", 10},
      {"fn main() { let xs = [1, 2, 3]; let s = 0; for v in xs { s = s + v; } s }",
       6},
      {"fn main() { let s = 0; for i in 0..10 { if i == 3 { break; } s = s + i; } "
       "s }",
       3},
      {"fn main() { let s = 0; for i in 0..5 { if i == 2 { continue; } s = s + i; "
       "} s }",
       8},
      {"fn main() { let nested = [[1, 2], 3]; let inner = nested[0]; len(inner) }",
       2},
      {"fn main() { let xs = []; push(xs, [1]); push(xs, \"a\"); len(xs) }", 2},
      {"fn main() { ord(\"A\") }", 65},
      {"fn main() { let s = \"ab\"; ord(s[0]) + ord(s[1]) }", 195},
      /* forward call: even before odd is compiled */
      {"fn even(n) { if n == 0 { 1 } else { odd(n - 1) } } fn odd(n) { if n == 0 { "
       "0 } else { even(n - 1) } } fn main() { even(4) * 10 + odd(3) }",
       11},
      {"fn main() { while true { break; } 7 }", 7},
      {"fn main() { let s = \"a\"; s = s + \"b\"; ord(s[0]) + ord(s[1]) }", 195},
      /* grow past strtab cap=16 — catches Program-by-value / double-free */
      {"fn main() { let i = 0; let s = \"\"; while i < 20 { s = s + \"x\"; i = i + 1; "
       "} len(s) }",
       20},
      /* void helper: trailing RET with empty stack → 0 */
      {"fn bump(x) { println(x); } fn main() { bump(1); 9 }", 9},
      /* Prophet Memory (no Rust) */
      {"fn main() { let m = memory_config(10, 8, 4); assert(remember(m, [9, 0, 9], "
       "80) == true); assert(remember(m, [1, 1, 1], 5) == false); let st = "
       "mem_stats(m); assert(st[0] == 1); consolidate(m); let p = foresee(m, [9, "
       "0, 0]); assert(len(p) == 3); 0 }",
       0},
      {"fn main() { let m = memory_config(10, 8, 4); remember(m, [1, 2, 3], 90); "
       "consolidate(m); save_mind(m, \"minds/_lite_roundtrip.km\"); let m2 = "
       "load_mind(\"minds/_lite_roundtrip.km\"); let st = mem_stats(m2); "
       "assert(st[1] >= 1); 0 }",
       0},
      /* Tape autograd (no Rust) — small lr so 1x1 matmul converges */
      {"fn main() { let w = t_from([1, 1], [0.0]); let x = t_from([1, 1], [2.0]); "
       "let y = t_from([1, 1], [4.0]); let i = 0; while i < 40 { ag_clear(); let "
       "wid = ag_param(w); let pred = ag_matmul(wid, ag_const(x)); let loss = "
       "ag_mse(pred, ag_const(y)); ag_backward(loss); w = ag_step(wid, 0.05); i = "
       "i + 1; } assert(round(t_get(t_matmul(w, x), 0) * 10.0) == 40); 0 }",
       0},
      /* Vision helpers */
      {"fn main() { let img = load_ppm(\"../examples/ml/assets/dot.ppm\"); let p = "
       "t_patch_mean(img, 1, 1); assert(len(t_shape(p)) == 3); let w = t_from([1, "
       "3], [0.0, 0.0, 0.0]); let x = t_reshape(t_mean(img), [3, 1]); let y = "
       "t_from([1, 1], [1.5]); let g = t_linear_grad(w, x, y); let l0 = t_mse(t_"
       "matmul(w, x), y); w = t_sub(w, t_scale(g, 0.5)); let l1 = t_mse(t_matmul(w, "
       "x), y); assert(l1 < l0); 0 }",
       0},
      /* log + tensor IO + write_file */
      {"fn main() { let t = t_from([2], [1.0, 2.718281828]); let l = t_log(t); "
       "assert(round(t_get(l, 0) * 100.0) == 0); assert(save_tensor(t, "
       "\"minds/_lite_t.kt\") == 1); assert(len(t_shape(load_tensor(\"minds/_lite_t."
       "kt\"))) == 1); assert(write_file(\"minds/_lite_w.txt\", \"hi\") == 1); "
       "assert(ord(read_file(\"minds/_lite_w.txt\")[0]) == 104); 0 }",
       0},
  };
  size_t n = sizeof(cases) / sizeof(cases[0]);
  for (size_t i = 0; i < n; i++) {
    int64_t got = run_lite(cases[i].src);
    if (got != cases[i].want) {
      fprintf(stderr, "FAIL case %zu: got %lld want %lld\n", i, (long long)got,
              (long long)cases[i].want);
      return 1;
    }
  }
  printf("kenga-lite selftest ok (%zu cases)\n", n);
  printf("bootstrap: Rust not required for lite dialect\n");
  return 0;
}

#include "generated/rt_cli.inc.c"
