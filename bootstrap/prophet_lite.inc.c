/* Prophet Memory for kenga-lite — CPU f64 residual MLP, no Rust.
 * Included into kenga_lite.c (single TU). */
#include <math.h>

#ifndef KENGA_PROPHET_INC
#define KENGA_PROPHET_INC

#define PL_EP_MAX 128
#define PL_CORE_MAX 64
#define PL_DIM_MAX 64

typedef struct {
  double pattern[PL_DIM_MAX];
  double target[PL_DIM_MAX];
  int has_target;
  int dim;
  int target_dim;
  double surprise;
  uint64_t ts_ms;
} PlEpisode;

typedef struct {
  double pattern[PL_DIM_MAX];
  int dim;
  double importance;
  uint64_t replays;
} PlCore;

typedef struct {
  int dim, hidden;
  double w1[PL_DIM_MAX * PL_DIM_MAX * 3]; /* h*d, h<=3*d */
  double b1[PL_DIM_MAX * 3];
  double w2[PL_DIM_MAX * PL_DIM_MAX * 3]; /* d*h */
  double b2[PL_DIM_MAX];
  double w1_lock[PL_DIM_MAX * PL_DIM_MAX * 3];
  double b1_lock[PL_DIM_MAX * 3];
  double w2_lock[PL_DIM_MAX * PL_DIM_MAX * 3];
  double b2_lock[PL_DIM_MAX];
  uint64_t steps;
} PlWorld;

typedef struct {
  PlEpisode ep[PL_EP_MAX];
  int nep;
  PlCore core[PL_CORE_MAX];
  int ncore;
  PlWorld model;
  double threshold;
  int ep_cap;
  int core_cap;
  double lr;
} ProphetMem;

typedef struct {
  ProphetMem *data;
  size_t len, cap;
} MemHeap;

static MemHeap g_mems;

static Value V_mem(int64_t h) {
  Value v;
  v.tag = TAG_MEMORY;
  v.payload = h;
  return v;
}

static int pl_hidden_for(int dim) {
  int h = dim * 3;
  if (h < 8) h = 8;
  if (h > 64) h = 64;
  return h;
}

static double pl_xavier(int fan_in, int fan_out, int i) {
  double s = sqrt(6.0 / (double)(fan_in + fan_out));
  double u = (double)((i * 1103515245 + 12345) % 10000) / 10000.0;
  return (u * 2.0 - 1.0) * s;
}

static void pl_wm_init(PlWorld *m, int dim) {
  if (dim < 1) dim = 1;
  if (dim > PL_DIM_MAX) dim = PL_DIM_MAX;
  memset(m, 0, sizeof(*m));
  m->dim = dim;
  m->hidden = pl_hidden_for(dim);
  {
    int d = m->dim, h = m->hidden, i;
    for (i = 0; i < h * d; i++) m->w1[i] = pl_xavier(d, h, i);
    for (i = 0; i < d * h; i++) m->w2[i] = pl_xavier(h, d, i + 17) * 0.25;
  }
}

static void pl_wm_ensure(PlWorld *m, int dim) {
  if (dim < 1) dim = 1;
  if (dim > PL_DIM_MAX) dim = PL_DIM_MAX;
  if (dim <= m->dim) return;
  {
    PlWorld old = *m;
    int od = old.dim, oh = old.hidden, nh, r, c;
    pl_wm_init(m, dim);
    nh = m->hidden;
    for (r = 0; r < oh && r < nh; r++) {
      for (c = 0; c < od; c++) {
        m->w1[r * m->dim + c] = old.w1[r * od + c];
        m->w1_lock[r * m->dim + c] = old.w1_lock[r * od + c];
      }
      m->b1[r] = old.b1[r];
      m->b1_lock[r] = old.b1_lock[r];
    }
    for (r = 0; r < od; r++) {
      for (c = 0; c < oh && c < nh; c++) {
        m->w2[r * nh + c] = old.w2[r * oh + c];
        m->w2_lock[r * nh + c] = old.w2_lock[r * oh + c];
      }
      m->b2[r] = old.b2[r];
      m->b2_lock[r] = old.b2_lock[r];
    }
    m->steps = old.steps;
  }
}

static void pl_predict(const ProphetMem *mem, const double *obs, int on, double *out, int *out_n);

static void pl_forward(const PlWorld *m, const double *x, int xn, double *h_out, double *y_out) {
  int d = m->dim, h = m->hidden, r, c;
  for (r = 0; r < h; r++) {
    double s = m->b1[r];
    for (c = 0; c < d; c++) s += m->w1[r * d + c] * (c < xn ? x[c] : 0.0);
    h_out[r] = tanh(s);
  }
  for (r = 0; r < d; r++) {
    double delta = m->b2[r];
    for (c = 0; c < h; c++) delta += m->w2[r * h + c] * h_out[c];
    y_out[r] = (r < xn ? x[r] : 0.0) + delta;
  }
}

static double pl_train_step(PlWorld *m, const double *x, int xn, const double *tgt, int tn,
                            double lr) {
  int need = xn > tn ? xn : tn;
  if (need < 1) need = 1;
  pl_wm_ensure(m, need);
  {
    double hh[64], yy[64], dy[64], dh[64];
    int d = m->dim, h = m->hidden, r, c, i;
    double loss = 0.0;
    pl_forward(m, x, xn, hh, yy);
    for (i = 0; i < d; i++) {
      double t = i < tn ? tgt[i] : 0.0;
      double err = t - yy[i];
      loss += err * err;
      dy[i] = err;
    }
    loss /= (double)d;
    for (c = 0; c < h; c++) {
      double s = 0.0;
      for (r = 0; r < d; r++) s += m->w2[r * h + c] * dy[r];
      dh[c] = s * (1.0 - hh[c] * hh[c]);
    }
    for (r = 0; r < d; r++) {
      for (c = 0; c < h; c++) {
        int idx = r * h + c;
        double g = dy[r] * hh[c];
        double eff = lr / (1.0 + m->w2_lock[idx]);
        m->w2[idx] += eff * g;
        m->w2_lock[idx] += 0.02 * fabs(g);
      }
      {
        double eff = lr / (1.0 + m->b2_lock[r]);
        m->b2[r] += eff * dy[r];
        m->b2_lock[r] += 0.02 * fabs(dy[r]);
      }
    }
    for (r = 0; r < h; r++) {
      for (c = 0; c < d; c++) {
        int idx = r * d + c;
        double xv = c < xn ? x[c] : 0.0;
        double g = dh[r] * xv;
        double eff = lr / (1.0 + m->w1_lock[idx]);
        m->w1[idx] += eff * g;
        m->w1_lock[idx] += 0.02 * fabs(g);
      }
      {
        double eff = lr / (1.0 + m->b1_lock[r]);
        m->b1[r] += eff * dh[r];
        m->b1_lock[r] += 0.02 * fabs(dh[r]);
      }
    }
    m->steps++;
    return loss;
  }
}

static double pl_surprise(const double *a, int an, const double *b, int bn) {
  int n = an > bn ? an : bn;
  int i;
  double sum = 0.0;
  if (n < 1) n = 1;
  for (i = 0; i < n; i++) {
    double x = i < an ? a[i] : 0.0;
    double y = i < bn ? b[i] : 0.0;
    double d = x - y;
    sum += d * d;
  }
  return sqrt(sum / (double)n);
}

static int pl_value_to_pat(Value v, ListHeap *lists, double *out, int *out_n) {
  if (v.tag == TAG_LIST) {
    ValA *L;
    size_t i;
    if (v.payload < 0 || (size_t)v.payload >= lists->len) die("bad list");
    L = &lists->data[v.payload];
    if (L->len > (size_t)PL_DIM_MAX) die("pattern too long");
    for (i = 0; i < L->len; i++) out[i] = to_f64(L->data[i], "pattern");
    *out_n = (int)L->len;
    return 0;
  }
  if (v.tag == TAG_I64 || v.tag == TAG_F64) {
    out[0] = to_f64(v, "pattern");
    *out_n = 1;
    return 0;
  }
  die("memory pattern expects list or number");
  return 1;
}

static Value pl_pat_to_list(const double *p, int n, ListHeap *lists) {
  ValA empty = {0};
  int64_t h;
  int i;
  h = (int64_t)lists->len;
  listh_push(lists, empty);
  for (i = 0; i < n; i++) vala_push(&lists->data[h], V_f64(p[i]));
  return V_list(h);
}

static ProphetMem *pl_get(int64_t h) {
  if (h < 0 || (size_t)h >= g_mems.len) die("bad memory handle");
  return &g_mems.data[h];
}

static int64_t pl_mem_new(double thr, int ep_cap, int core_cap) {
  ProphetMem m;
  memset(&m, 0, sizeof(m));
  m.threshold = thr;
  m.ep_cap = ep_cap < 1 ? 1 : (ep_cap > PL_EP_MAX ? PL_EP_MAX : ep_cap);
  m.core_cap = core_cap < 1 ? 1 : (core_cap > PL_CORE_MAX ? PL_CORE_MAX : core_cap);
  m.lr = 0.08;
  pl_wm_init(&m.model, 1);
  if (g_mems.len + 1 > g_mems.cap) {
    g_mems.cap = g_mems.cap ? g_mems.cap * 2 : 4;
    g_mems.data = (ProphetMem *)xrealloc(g_mems.data, g_mems.cap * sizeof(ProphetMem));
  }
  g_mems.data[g_mems.len] = m;
  return (int64_t)g_mems.len++;
}

static void pl_mems_reset(void) {
  free(g_mems.data);
  g_mems.data = NULL;
  g_mems.len = g_mems.cap = 0;
}

static int pl_remember(ProphetMem *mem, const double *pat, int dim, double surprise) {
  int i, min_i;
  double min_s;
  if (surprise < mem->threshold) return 0;
  pl_wm_ensure(&mem->model, dim);
  if (mem->nep >= mem->ep_cap) {
    min_i = 0;
    min_s = mem->ep[0].surprise;
    for (i = 1; i < mem->nep; i++) {
      if (mem->ep[i].surprise < min_s) {
        min_s = mem->ep[i].surprise;
        min_i = i;
      }
    }
    mem->ep[min_i] = mem->ep[mem->nep - 1];
    mem->nep--;
  }
  {
    PlEpisode *e = &mem->ep[mem->nep++];
    memset(e, 0, sizeof(*e));
    e->dim = dim;
    e->surprise = surprise;
    e->has_target = 0;
    e->ts_ms = 0;
    for (i = 0; i < dim; i++) e->pattern[i] = pat[i];
  }
  return 1;
}

static int pl_remember_pair(ProphetMem *mem, const double *pat, int dim, const double *tgt,
                            int td, double surprise, uint64_t ts_ms) {
  int i, min_i;
  double min_s;
  if (surprise < mem->threshold) return 0;
  pl_wm_ensure(&mem->model, dim > td ? dim : td);
  if (mem->nep >= mem->ep_cap) {
    min_i = 0;
    min_s = mem->ep[0].surprise;
    for (i = 1; i < mem->nep; i++) {
      if (mem->ep[i].surprise < min_s) {
        min_s = mem->ep[i].surprise;
        min_i = i;
      }
    }
    mem->ep[min_i] = mem->ep[mem->nep - 1];
    mem->nep--;
  }
  {
    PlEpisode *e = &mem->ep[mem->nep++];
    memset(e, 0, sizeof(*e));
    e->dim = dim;
    e->surprise = surprise;
    e->ts_ms = ts_ms;
    e->has_target = tgt && td > 0;
    for (i = 0; i < dim; i++) e->pattern[i] = pat[i];
    if (e->has_target) {
      int n = td < PL_DIM_MAX ? td : PL_DIM_MAX;
      e->target_dim = n;
      for (i = 0; i < n; i++) e->target[i] = tgt[i];
    }
  }
  return 1;
}

static double pl_learn(ProphetMem *mem, const double *x, int xn, const double *y, int yn) {
  return pl_train_step(&mem->model, x, xn, y, yn, mem->lr);
}

static void pl_unroll(const ProphetMem *mem, const double *obs, int on, int steps, double *traj,
                      int *out_steps, int *out_dim) {
  double cur[PL_DIM_MAX], nxt[PL_DIM_MAX];
  int cn = 0, nn = 0, s, i;
  if (steps < 1) steps = 1;
  if (steps > 64) steps = 64;
  for (i = 0; i < on && i < PL_DIM_MAX; i++) cur[i] = obs[i];
  cn = on < PL_DIM_MAX ? on : PL_DIM_MAX;
  for (s = 0; s < steps; s++) {
    pl_predict(mem, cur, cn, nxt, &nn);
    for (i = 0; i < nn; i++) traj[s * PL_DIM_MAX + i] = nxt[i];
    for (i = 0; i < nn; i++) cur[i] = nxt[i];
    cn = nn;
  }
  *out_steps = steps;
  *out_dim = cn;
}

static double pl_train_physics_epoch(ProphetMem *mem) {
  double loss = 0.0, n = 0.0;
  int pos, vel;
  for (pos = 0; pos < 14; pos++) {
    for (vel = 0; vel < 3; vel++) {
      int v = vel - 1;
      int fuel = 9;
      double x[3], y[3];
      x[0] = (double)pos;
      x[1] = (double)v;
      x[2] = (double)fuel;
      y[0] = (double)(pos + v);
      y[1] = (double)v;
      y[2] = (double)(fuel - 1);
      pl_remember_pair(mem, x, 3, y, 3, 0.5, 0);
      loss += pl_learn(mem, x, 3, y, 3);
      n += 1.0;
    }
  }
  return loss / (n > 1.0 ? n : 1.0);
}

static void pl_write_f64s(FILE *f, const double *xs, int n) {
  int i;
  for (i = 0; i < n; i++) {
    if (i) fputc(' ', f);
    fprintf(f, "%.17g", xs[i]);
  }
  fputc('\n', f);
}

static void pl_ensure_parent(const char *path) {
  char buf[1024];
  size_t i, n = strlen(path);
  if (n == 0 || n >= sizeof(buf)) return;
  memcpy(buf, path, n + 1);
  for (i = 1; i < n; i++) {
    if (buf[i] == '/' || buf[i] == '\\') {
      char c = buf[i];
      buf[i] = 0;
#ifdef _WIN32
      _mkdir(buf);
#else
      mkdir(buf, 0755);
#endif
      buf[i] = c;
    }
  }
}

static int pl_save_mind(const ProphetMem *mem, const char *path) {
  FILE *f;
  int d, h, i;
  pl_ensure_parent(path);
  f = fopen(path, "wb");
  if (!f) return 0;
  d = mem->model.dim;
  h = mem->model.hidden;
  fprintf(f, "KENGA_MIND 1\n");
  fprintf(f, "threshold %.17g\n", mem->threshold);
  fprintf(f, "ep_cap %d\n", mem->ep_cap);
  fprintf(f, "core_cap %d\n", mem->core_cap);
  fprintf(f, "lr %.17g\n", mem->lr);
  fprintf(f, "model %d %d %llu\n", d, h, (unsigned long long)mem->model.steps);
  pl_write_f64s(f, mem->model.w1, h * d);
  pl_write_f64s(f, mem->model.b1, h);
  pl_write_f64s(f, mem->model.w2, d * h);
  pl_write_f64s(f, mem->model.b2, d);
  pl_write_f64s(f, mem->model.w1_lock, h * d);
  pl_write_f64s(f, mem->model.b1_lock, h);
  pl_write_f64s(f, mem->model.w2_lock, d * h);
  pl_write_f64s(f, mem->model.b2_lock, d);
  fprintf(f, "core %d\n", mem->ncore);
  for (i = 0; i < mem->ncore; i++) {
    fprintf(f, "%.17g %llu %d\n", mem->core[i].importance,
            (unsigned long long)mem->core[i].replays, mem->core[i].dim);
    pl_write_f64s(f, mem->core[i].pattern, mem->core[i].dim);
  }
  fprintf(f, "episodic %d\n", mem->nep);
  for (i = 0; i < mem->nep; i++) {
    int has_t = mem->ep[i].has_target ? 1 : 0;
    int tlen = has_t ? (mem->ep[i].target_dim > 0 ? mem->ep[i].target_dim : mem->ep[i].dim) : 0;
    fprintf(f, "%.17g %llu %d %d %d\n", mem->ep[i].surprise,
            (unsigned long long)mem->ep[i].ts_ms, mem->ep[i].dim, has_t, tlen);
    pl_write_f64s(f, mem->ep[i].pattern, mem->ep[i].dim);
    if (has_t) pl_write_f64s(f, mem->ep[i].target, tlen);
  }
  fclose(f);
  return 1;
}

static int pl_parse_f64s_line(const char *line, double *out, int maxn, int expect) {
  int n = 0;
  const char *p = line;
  while (*p && n < maxn) {
    char *end = NULL;
    double v;
    while (*p == ' ' || *p == '\t') p++;
    if (!*p || *p == '\n' || *p == '\r') break;
    v = strtod(p, &end);
    if (end == p) return -1;
    out[n++] = v;
    p = end;
  }
  if (expect >= 0 && n != expect) return -1;
  return n;
}

static char *pl_read_file(const char *path, size_t *out_len) {
  FILE *f = fopen(path, "rb");
  long sz;
  char *buf;
  if (!f) return NULL;
  if (fseek(f, 0, SEEK_END) != 0) {
    fclose(f);
    return NULL;
  }
  sz = ftell(f);
  if (sz < 0) {
    fclose(f);
    return NULL;
  }
  rewind(f);
  buf = (char *)malloc((size_t)sz + 1);
  if (!buf) {
    fclose(f);
    return NULL;
  }
  if (fread(buf, 1, (size_t)sz, f) != (size_t)sz) {
    free(buf);
    fclose(f);
    return NULL;
  }
  buf[sz] = 0;
  fclose(f);
  if (out_len) *out_len = (size_t)sz;
  return buf;
}

/* Split non-empty lines into pointers into mutable buffer (newlines → 0). */
static int pl_split_lines(char *raw, char **lines, int max_lines) {
  int n = 0;
  char *p = raw;
  while (*p && n < max_lines) {
    char *start;
    while (*p == '\r' || *p == '\n') p++;
    if (!*p) break;
    start = p;
    while (*p && *p != '\n' && *p != '\r') p++;
    if (*p) {
      *p = 0;
      p++;
    }
    if (start[0]) lines[n++] = start;
  }
  return n;
}

static int64_t pl_load_mind(const char *path) {
  char *raw;
  char *lines[4096];
  int nlines, li = 0;
  ProphetMem m;
  int d, h, i, core_n, ep_n;
  unsigned long long steps;
  memset(&m, 0, sizeof(m));
  raw = pl_read_file(path, NULL);
  if (!raw) die("cannot read mind file");
  nlines = pl_split_lines(raw, lines, 4096);
  if (nlines < 6 || strcmp(lines[0], "KENGA_MIND 1") != 0) {
    free(raw);
    die("unsupported mind format");
  }
  if (sscanf(lines[1], "threshold %lf", &m.threshold) != 1) die("bad threshold");
  if (sscanf(lines[2], "ep_cap %d", &m.ep_cap) != 1) die("bad ep_cap");
  if (sscanf(lines[3], "core_cap %d", &m.core_cap) != 1) die("bad core_cap");
  if (sscanf(lines[4], "lr %lf", &m.lr) != 1) die("bad lr");
  if (sscanf(lines[5], "model %d %d %llu", &d, &h, &steps) != 3) die("bad model");
  if (d < 1 || d > PL_DIM_MAX || h < 1 || h > PL_DIM_MAX * 3) die("model dims too large for lite");
  if (m.ep_cap > PL_EP_MAX) m.ep_cap = PL_EP_MAX;
  if (m.core_cap > PL_CORE_MAX) m.core_cap = PL_CORE_MAX;
  memset(&m.model, 0, sizeof(m.model));
  m.model.dim = d;
  m.model.hidden = h;
  m.model.steps = (uint64_t)steps;
  li = 6;
#define PL_TAKE(dst, count, what)                                                                  \
  do {                                                                                             \
    int _got;                                                                                      \
    if (li >= nlines) {                                                                            \
      free(raw);                                                                                   \
      die("missing " what);                                                                        \
    }                                                                                              \
    _got = pl_parse_f64s_line(lines[li++], (dst), (count), (count));                               \
    if (_got < 0) {                                                                                \
      free(raw);                                                                                   \
      die("bad " what);                                                                            \
    }                                                                                              \
  } while (0)
  PL_TAKE(m.model.w1, h * d, "w1");
  PL_TAKE(m.model.b1, h, "b1");
  PL_TAKE(m.model.w2, d * h, "w2");
  PL_TAKE(m.model.b2, d, "b2");
  PL_TAKE(m.model.w1_lock, h * d, "w1_lock");
  PL_TAKE(m.model.b1_lock, h, "b1_lock");
  PL_TAKE(m.model.w2_lock, d * h, "w2_lock");
  PL_TAKE(m.model.b2_lock, d, "b2_lock");
#undef PL_TAKE
  if (li >= nlines || sscanf(lines[li++], "core %d", &core_n) != 1) {
    free(raw);
    die("bad core header");
  }
  if (core_n < 0 || core_n > PL_CORE_MAX) {
    free(raw);
    die("too many core traces");
  }
  m.ncore = core_n;
  for (i = 0; i < core_n; i++) {
    unsigned long long rep;
    int plen;
    if (li >= nlines) {
      free(raw);
      die("missing core meta");
    }
    if (sscanf(lines[li++], "%lf %llu %d", &m.core[i].importance, &rep, &plen) != 3) {
      free(raw);
      die("bad core meta");
    }
    m.core[i].replays = (uint64_t)rep;
    if (plen < 1 || plen > PL_DIM_MAX) {
      free(raw);
      die("bad core plen");
    }
    m.core[i].dim = plen;
    if (li >= nlines || pl_parse_f64s_line(lines[li++], m.core[i].pattern, plen, plen) < 0) {
      free(raw);
      die("bad core pattern");
    }
  }
  if (li >= nlines || sscanf(lines[li++], "episodic %d", &ep_n) != 1) {
    free(raw);
    die("bad episodic header");
  }
  if (ep_n < 0 || ep_n > PL_EP_MAX) {
    free(raw);
    die("too many episodes");
  }
  m.nep = ep_n;
  for (i = 0; i < ep_n; i++) {
    unsigned long long ts;
    int plen, has_t, tlen;
    if (li >= nlines) {
      free(raw);
      die("missing episode meta");
    }
    if (sscanf(lines[li++], "%lf %llu %d %d %d", &m.ep[i].surprise, &ts, &plen, &has_t, &tlen) !=
        5) {
      free(raw);
      die("bad episode meta");
    }
    m.ep[i].ts_ms = (uint64_t)ts;
    m.ep[i].has_target = has_t ? 1 : 0;
    if (plen < 1 || plen > PL_DIM_MAX) {
      free(raw);
      die("bad episode plen");
    }
    m.ep[i].dim = plen;
    if (li >= nlines || pl_parse_f64s_line(lines[li++], m.ep[i].pattern, plen, plen) < 0) {
      free(raw);
      die("bad episode pattern");
    }
    if (has_t) {
      if (tlen < 1 || tlen > PL_DIM_MAX) {
        free(raw);
        die("bad episode tlen");
      }
      m.ep[i].target_dim = tlen;
      if (li >= nlines || pl_parse_f64s_line(lines[li++], m.ep[i].target, tlen, tlen) < 0) {
        free(raw);
        die("bad episode target");
      }
    }
  }
  free(raw);
  if (g_mems.len + 1 > g_mems.cap) {
    g_mems.cap = g_mems.cap ? g_mems.cap * 2 : 4;
    g_mems.data = (ProphetMem *)xrealloc(g_mems.data, g_mems.cap * sizeof(ProphetMem));
  }
  g_mems.data[g_mems.len] = m;
  return (int64_t)g_mems.len++;
}

static int pl_nearest_core(const ProphetMem *mem, const double *pat, int dim, double *out_dist) {
  int i, best = -1;
  double best_d = 1e300;
  for (i = 0; i < mem->ncore; i++) {
    double d = pl_surprise(mem->core[i].pattern, mem->core[i].dim, pat, dim);
    if (d < best_d) {
      best_d = d;
      best = i;
    }
  }
  if (out_dist) *out_dist = best_d;
  return best;
}

static void pl_blend_core(const ProphetMem *mem, const double *obs, int on, double *out, int *out_n) {
  int i, t, dim = on;
  double wsum = 0.0;
  double ws[PL_CORE_MAX];
  int idx[PL_CORE_MAX];
  int n = mem->ncore;
  if (n == 0) {
    *out_n = 0;
    return;
  }
  for (i = 0; i < n; i++) {
    double d = pl_surprise(mem->core[i].pattern, mem->core[i].dim, obs, on);
    ws[i] = (1.0 / (0.05 + d)) * (1.0 + mem->core[i].importance);
    idx[i] = i;
    if (mem->core[i].dim > dim) dim = mem->core[i].dim;
  }
  /* selection sort top-3 by weight */
  for (i = 0; i < n; i++) {
    int j;
    for (j = i + 1; j < n; j++) {
      if (ws[idx[j]] > ws[idx[i]]) {
        int tmp = idx[i];
        idx[i] = idx[j];
        idx[j] = tmp;
      }
    }
  }
  t = n < 3 ? n : 3;
  for (i = 0; i < dim; i++) out[i] = 0.0;
  for (i = 0; i < t; i++) {
    int ci = idx[i];
    int k;
    wsum += ws[ci];
    for (k = 0; k < dim; k++) {
      double p = k < mem->core[ci].dim ? mem->core[ci].pattern[k] : 0.0;
      out[k] += ws[ci] * p;
    }
  }
  if (wsum < 1e-9) wsum = 1e-9;
  for (i = 0; i < dim; i++) out[i] /= wsum;
  *out_n = dim;
}

static void pl_predict(const ProphetMem *mem, const double *obs, int on, double *out, int *out_n) {
  PlWorld tmp = mem->model;
  double hh[64];
  pl_wm_ensure(&tmp, on > 0 ? on : 1);
  pl_forward(&tmp, obs, on, hh, out);
  *out_n = tmp.dim;
}

static void pl_foresee(const ProphetMem *mem, const double *obs, int on, double *out, int *out_n) {
  double neural[PL_DIM_MAX], core[PL_DIM_MAX];
  int nn = 0, cn = 0, dim, i;
  double nw, cw;
  pl_predict(mem, obs, on, neural, &nn);
  pl_blend_core(mem, obs, on, core, &cn);
  if (cn == 0) {
    if (mem->model.steps == 0) {
      for (i = 0; i < on; i++) out[i] = obs[i];
      *out_n = on;
    } else {
      for (i = 0; i < nn; i++) out[i] = neural[i];
      *out_n = nn;
    }
    return;
  }
  dim = nn > cn ? nn : cn;
  if (on > dim) dim = on;
  nw = mem->model.steps > 0 ? 0.6 : 0.15;
  cw = 1.0 - nw;
  for (i = 0; i < dim; i++) {
    double n = i < nn ? neural[i] : 0.0;
    double c = i < cn ? core[i] : 0.0;
    double o = i < on ? obs[i] : 0.0;
    out[i] = nw * n + cw * c * 0.8 + o * 0.2;
  }
  *out_n = dim;
}

static int64_t pl_consolidate(ProphetMem *mem) {
  int folded = 0;
  int e;
  int nep = mem->nep;
  PlEpisode eps[PL_EP_MAX];
  memcpy(eps, mem->ep, sizeof(eps[0]) * (size_t)nep);
  mem->nep = 0;
  for (e = 0; e < nep; e++) {
    PlEpisode *ep = &eps[e];
    const double *tgt = ep->has_target ? ep->target : ep->pattern;
    int td = ep->has_target ? (ep->target_dim > 0 ? ep->target_dim : ep->dim) : ep->dim;
    double dist = 0.0;
    int ni;
    folded++;
    pl_wm_ensure(&mem->model, ep->dim > td ? ep->dim : td);
    pl_train_step(&mem->model, ep->pattern, ep->dim, tgt, td, mem->lr);
    if (ep->surprise > 0.4)
      pl_train_step(&mem->model, ep->pattern, ep->dim, tgt, td, mem->lr * 0.5);
    ni = pl_nearest_core(mem, ep->pattern, ep->dim, &dist);
    if (ni >= 0 && dist < 0.35) {
      double lock = mem->core[ni].importance;
      double lr = 0.35 / (1.0 + (lock > 0.0 ? lock : 0.0));
      int dim = mem->core[ni].dim > ep->dim ? mem->core[ni].dim : ep->dim;
      int i;
      if (dim > PL_DIM_MAX) dim = PL_DIM_MAX;
      for (i = 0; i < dim; i++) {
        double old = i < mem->core[ni].dim ? mem->core[ni].pattern[i] : 0.0;
        double nw = i < ep->dim ? ep->pattern[i] : old;
        mem->core[ni].pattern[i] = old * (1.0 - lr) + nw * lr;
      }
      mem->core[ni].dim = dim;
      mem->core[ni].importance += ep->surprise * 0.5;
      mem->core[ni].replays++;
      continue;
    }
    if (mem->ncore < mem->core_cap) {
      PlCore *c = &mem->core[mem->ncore++];
      int i;
      memset(c, 0, sizeof(*c));
      c->dim = ep->dim;
      c->importance = ep->surprise;
      c->replays = 1;
      for (i = 0; i < ep->dim; i++) c->pattern[i] = ep->pattern[i];
    }
  }
  if (mem->ncore > mem->core_cap) {
    /* drop lowest importance */
    while (mem->ncore > mem->core_cap) {
      int i, worst = 0;
      for (i = 1; i < mem->ncore; i++)
        if (mem->core[i].importance < mem->core[worst].importance) worst = i;
      mem->core[worst] = mem->core[mem->ncore - 1];
      mem->ncore--;
    }
  }
  return folded;
}

static Value pl_recall(ProphetMem *mem, const double *q, int qn, int k, ListHeap *lists) {
  typedef struct {
    double d;
    int dim;
    double p[PL_DIM_MAX];
  } Hit;
  Hit hits[PL_EP_MAX + PL_CORE_MAX];
  int nh = 0, i, j;
  ValA outer = {0};
  int64_t oh;
  if (k < 1) k = 1;
  for (i = 0; i < mem->ncore && nh < (int)(sizeof hits / sizeof hits[0]); i++) {
    hits[nh].d = pl_surprise(mem->core[i].pattern, mem->core[i].dim, q, qn);
    hits[nh].dim = mem->core[i].dim;
    memcpy(hits[nh].p, mem->core[i].pattern, sizeof(double) * (size_t)mem->core[i].dim);
    nh++;
  }
  for (i = 0; i < mem->nep && nh < (int)(sizeof hits / sizeof hits[0]); i++) {
    hits[nh].d = pl_surprise(mem->ep[i].pattern, mem->ep[i].dim, q, qn);
    hits[nh].dim = mem->ep[i].dim;
    memcpy(hits[nh].p, mem->ep[i].pattern, sizeof(double) * (size_t)mem->ep[i].dim);
    nh++;
  }
  for (i = 0; i < nh; i++) {
    for (j = i + 1; j < nh; j++) {
      if (hits[j].d < hits[i].d) {
        Hit t = hits[i];
        hits[i] = hits[j];
        hits[j] = t;
      }
    }
  }
  if (k > nh) k = nh;
  oh = (int64_t)lists->len;
  listh_push(lists, outer);
  for (i = 0; i < k; i++) {
    Value inner = pl_pat_to_list(hits[i].p, hits[i].dim, lists);
    vala_push(&lists->data[oh], inner);
  }
  return V_list(oh);
}

static Value pl_stats(ProphetMem *mem, ListHeap *lists) {
  ValA empty = {0};
  int64_t h = (int64_t)lists->len;
  int locked = 0, i;
  for (i = 0; i < mem->ncore; i++)
    if (mem->core[i].importance >= 1.0) locked++;
  listh_push(lists, empty);
  vala_push(&lists->data[h], V_i64(mem->nep));
  vala_push(&lists->data[h], V_i64(mem->ncore));
  vala_push(&lists->data[h], V_i64(locked));
  vala_push(&lists->data[h], V_i64((int64_t)mem->model.steps));
  vala_push(&lists->data[h], V_i64(mem->model.dim));
  vala_push(&lists->data[h], V_i64(mem->model.hidden));
  return V_list(h);
}

#endif /* KENGA_PROPHET_INC */
