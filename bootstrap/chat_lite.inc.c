/* Interactive Prophet chat for kenga-lite — no Rust.
 * Included into kenga_lite.c after prophet_lite.inc.c */
#ifndef KENGA_CHAT_LITE_INC
#define KENGA_CHAT_LITE_INC

#include <stdarg.h>

static double g_last_obs[PL_DIM_MAX];
static int g_last_obs_n;

static int cl_contains(const char *hay, const char *needle) {
  return needle && needle[0] && strstr(hay, needle) != NULL;
}

static int cl_any(const char *hay, ...) {
  va_list ap;
  const char *n;
  int hit = 0;
  va_start(ap, hay);
  while ((n = va_arg(ap, const char *)) != NULL) {
    if (cl_contains(hay, n)) {
      hit = 1;
      break;
    }
  }
  va_end(ap);
  return hit;
}

static void cl_lower(char *s) {
  for (; *s; s++) {
    if ((unsigned char)*s < 128) *s = (char)tolower((unsigned char)*s);
  }
}

static int cl_extract_nums(const char *s, double *out, int maxn) {
  int n = 0;
  const char *p = s;
  while (*p && n < maxn) {
    while (*p && !((*p >= '0' && *p <= '9') || *p == '-' || *p == '.')) p++;
    if (!*p) break;
    {
      char *end = NULL;
      double v = strtod(p, &end);
      if (end == p) {
        p++;
        continue;
      }
      out[n++] = v;
      p = end;
    }
  }
  return n;
}

static void cl_fmt_round(const double *xs, int n, char *buf, size_t cap) {
  size_t o = 0;
  int i;
  buf[0] = 0;
  for (i = 0; i < n; i++) {
    int w = snprintf(buf + o, cap > o ? cap - o : 0, "%s%lld", i ? " " : "",
                     (long long)llround(xs[i]));
    if (w < 0) break;
    o += (size_t)w;
    if (o >= cap) break;
  }
}

static void cl_print_help(void) {
  printf("разговор:  привет | ты кто | что умеешь | поговорим | честно\n");
  printf("модель:    смотри 5 1 6 | что будет через 4 | обучи 10\n");
  printf("память:    научи 5 1 6 -> 6 1 5 | вспомни … | спи | сохрани\n");
  printf("выход:     выход / quit\n");
  printf("(lite, без Rust)\n");
}

static int cl_handle(ProphetMem **mind_slot, const char *path, const char *line) {
  char lower[1024];
  double nums[PL_DIM_MAX];
  int nn;
  ProphetMem *mind = *mind_slot;
  size_t L;
  if (!line) return 1;
  L = strlen(line);
  if (L >= sizeof(lower)) L = sizeof(lower) - 1;
  memcpy(lower, line, L);
  lower[L] = 0;
  cl_lower(lower);
  nn = cl_extract_nums(line, nums, PL_DIM_MAX);

  if (cl_any(lower, "quit", "exit", "выход", "пока", NULL)) {
    printf("пока — mind на диске, если сохранил\n");
    return 0;
  }
  if (cl_any(lower, "привет", "hello", "hi", "здравств", "hey", NULL)) {
    printf("привет. я живой world-model Kenga на lite (steps=%llu, dim=%d).\n",
           (unsigned long long)mind->model.steps, mind->model.dim);
    if (mind->model.dim >= 9)
      printf("этот mind — multimodal: картинка+звук → вектор → предсказание.\n");
    else
      printf("этот mind — физика агента [pos vel fuel].\n");
    printf("можешь: «что умеешь», «поговорим», «смотри …», «помощь».\n");
    return 1;
  }
  if (cl_any(lower, "что умеешь", "что ты умеешь", "что можешь", "возможности", "capabilities", NULL)) {
    printf("умею сейчас (lite):\n");
    printf("  • предсказывать следующий вектор (predict / unroll)\n");
    printf("  • удивляться и помнить эпизоды; спать (consolidate)\n");
    printf("  • save/load .km без Rust\n");
    if (mind->model.dim >= 9) printf("  • multimodal obs dim=%d\n", mind->model.dim);
    printf("пока не умею: свободный диалог как у большой LLM.\n");
    return 1;
  }
  if (cl_any(lower, "поговори", "поговорим", "просто поговорить", "хочу поговорить", "давай болтать", NULL)) {
    printf("давай. я говорю о том, что помню телом модели.\n");
    printf("сейчас: steps=%llu dim=%d core=%d.\n", (unsigned long long)mind->model.steps,
           mind->model.dim, mind->ncore);
    printf("если нужна честность про потолок — «честно».\n");
    return 1;
  }
  if (cl_any(lower, "ты кто", "кто ты", "расскажи о себе", "о себе", "what are you", "who are you", NULL) ||
      strcmp(lower, "что ты") == 0 || strncmp(lower, "что ты такое", 12) == 0) {
    printf("я Prophet-mind: residual MLP, dim=%d, hidden=%d, steps=%llu.\n", mind->model.dim,
           mind->model.hidden, (unsigned long long)mind->model.steps);
    printf("episodic=%d, core=%d. sense→learn→sleep.\n", mind->nep, mind->ncore);
    if (mind->model.dim >= 9) printf("обучен на PPM+WAV (living multimodal).\n");
    printf("я ещё не чат-LLM — но живу в цикле на C-lite.\n");
    return 1;
  }
  if (cl_any(lower, "честно", "ты ии", "ты llm", "как grok", "как gpt", "нейросеть", "языковая модель", "это всё", NULL)) {
    printf("честно: сейчас я world-model (векторы → векторы), не языковая модель.\n");
    printf("«умная как Grok» = tiny LM → данные → f32/GPU. живой цикл уже есть.\n");
    return 1;
  }
  if (strcmp(lower, "?") == 0 || strcmp(lower, "help") == 0 || strcmp(lower, "помощь") == 0 ||
      strncmp(lower, "help ", 5) == 0 || strncmp(lower, "помощь", 6) == 0) {
    cl_print_help();
    return 1;
  }
  if (cl_any(lower, "status", "статус", "как дела", "stats", NULL)) {
    printf("я помню: ep=%d core=%d steps=%llu dim=%dx%d (lr=%g)\n", mind->nep, mind->ncore,
           (unsigned long long)mind->model.steps, mind->model.dim, mind->model.hidden, mind->lr);
    return 1;
  }
  if (cl_any(lower, "sleep", "спи", "консолид", "засни", NULL)) {
    printf("сон: в ядро ушло %lld\n", (long long)pl_consolidate(mind));
    return 1;
  }
  if (cl_any(lower, "save", "сохрани", "запиши mind", NULL)) {
    if (!pl_save_mind(mind, path))
      printf("не смог сохранить %s\n", path);
    else
      printf("сохранил %s\n", path);
    return 1;
  }
  if (cl_any(lower, "load", "загрузи", NULL)) {
    int64_t h = pl_load_mind(path);
    *mind_slot = pl_get(h);
    printf("загрузил %s\n", path);
    return 1;
  }
  if (cl_any(lower, "train", "обучи", "тренир", "учись", NULL)) {
    int epochs = nn > 0 ? (int)nums[0] : 8;
    int e;
    if (epochs < 1) epochs = 1;
    if (epochs > 64) epochs = 64;
    printf("учусь %d эпох…\n", epochs);
    for (e = 0; e < epochs; e++) {
      double loss = pl_train_physics_epoch(mind);
      printf("  эпоха %d: loss=%.6f\n", e, loss);
    }
    printf("поспал, сложил эпизодов: %lld\n", (long long)pl_consolidate(mind));
    return 1;
  }
  if (cl_any(lower, "future", "будущ", "через", "что будет", "unroll", "предскажи", "завтра", "скоро", "дальше", NULL)) {
    double obs[PL_DIM_MAX], traj[64 * PL_DIM_MAX];
    int on = 0, steps = 4, os = 0, od = 0, i;
    char buf[256];
    if (cl_contains(lower, "завтра") || cl_contains(lower, "скоро")) steps = 1;
    if (nn >= 4) {
      steps = (int)nums[nn - 1];
      on = nn - 1;
      for (i = 0; i < on; i++) obs[i] = nums[i];
    } else if (nn == 1 && !cl_contains(lower, "смотри")) {
      steps = (int)nums[0];
      on = g_last_obs_n;
      for (i = 0; i < on; i++) obs[i] = g_last_obs[i];
      if (on < 1) {
        printf("сначала «смотри 5 1 6», потом «что будет…»\n");
        return 1;
      }
    } else if (nn >= 3) {
      on = nn;
      for (i = 0; i < on; i++) obs[i] = nums[i];
    } else {
      on = g_last_obs_n;
      for (i = 0; i < on; i++) obs[i] = g_last_obs[i];
      if (on < 1) {
        printf("usage: future n | future a b c n\n");
        return 1;
      }
    }
    if (steps < 1) steps = 1;
    pl_unroll(mind, obs, on, steps, traj, &os, &od);
    cl_fmt_round(obs, on, buf, sizeof(buf));
    printf("если старт %s, то через %d шагов:\n", buf, steps);
    for (i = 0; i < os; i++) {
      cl_fmt_round(traj + i * PL_DIM_MAX, od, buf, sizeof(buf));
      printf("  +%d → %s\n", i + 1, buf);
    }
    g_last_obs_n = on;
    for (i = 0; i < on; i++) g_last_obs[i] = obs[i];
    return 1;
  }
  if (cl_any(lower, "see", "смотри", "наблюд", "сейчас", "состояние", "sense", NULL) ||
      (nn > 0 && line[strspn(line, "0123456789.-+eE \t")] == 0)) {
    double pred[PL_DIM_MAX], fore[PL_DIM_MAX];
    int pn = 0, fn = 0, i, on = nn;
    char buf[256];
    double s;
    if (on < 1) {
      cl_print_help();
      return 1;
    }
    pl_predict(mind, nums, on, pred, &pn);
    pl_foresee(mind, nums, on, fore, &fn);
    s = pl_surprise(pred, pn, nums, on);
    cl_fmt_round(nums, on, buf, sizeof(buf));
    printf("сейчас     %s\n", buf);
    cl_fmt_round(pred, pn, buf, sizeof(buf));
    printf("я жду      %s\n", buf);
    cl_fmt_round(fore, fn, buf, sizeof(buf));
    printf("гибрид     %s\n", buf);
    if (s > 0.5)
      printf("это для меня странновато (surprise=%.3f)\n", s);
    else
      printf("похоже на знакомое (surprise=%.3f)\n", s);
    g_last_obs_n = on;
    for (i = 0; i < on; i++) g_last_obs[i] = nums[i];
    return 1;
  }
  if (cl_any(lower, "teach", "научи", "запомни", "выучи", NULL) ||
      cl_contains(line, "->") || cl_contains(line, "→")) {
    int mid, i;
    double loss;
    char a[128], b[128];
    if (nn < 2) {
      printf("usage: научи 5 1 6 -> 6 1 5\n");
      return 1;
    }
    mid = nn / 2;
    pl_remember_pair(mind, nums, mid, nums + mid, nn - mid, 0.6, 0);
    loss = pl_learn(mind, nums, mid, nums + mid, nn - mid);
    cl_fmt_round(nums, mid, a, sizeof(a));
    cl_fmt_round(nums + mid, nn - mid, b, sizeof(b));
    printf("запомнил %s → %s  (loss=%.4f)\n", a, b, loss);
    (void)i;
    return 1;
  }
  if (cl_any(lower, "recall", "вспомни", "похож", NULL)) {
    /* reuse pl_recall via temporary list heap — print patterns simply */
    double hits_d[16];
    int hi, k = 3, i, j;
    typedef struct {
      double d;
      int dim;
      double p[PL_DIM_MAX];
    } Hit;
    Hit hits[PL_EP_MAX + PL_CORE_MAX];
    int nh = 0;
    char buf[256];
    if (nn < 1) {
      cl_print_help();
      return 1;
    }
    for (i = 0; i < mind->ncore; i++) {
      hits[nh].d = pl_surprise(mind->core[i].pattern, mind->core[i].dim, nums, nn);
      hits[nh].dim = mind->core[i].dim;
      for (j = 0; j < hits[nh].dim; j++) hits[nh].p[j] = mind->core[i].pattern[j];
      nh++;
    }
    for (i = 0; i < mind->nep; i++) {
      hits[nh].d = pl_surprise(mind->ep[i].pattern, mind->ep[i].dim, nums, nn);
      hits[nh].dim = mind->ep[i].dim;
      for (j = 0; j < hits[nh].dim; j++) hits[nh].p[j] = mind->ep[i].pattern[j];
      nh++;
    }
    for (i = 0; i < nh; i++)
      for (j = i + 1; j < nh; j++)
        if (hits[j].d < hits[i].d) {
          Hit t = hits[i];
          hits[i] = hits[j];
          hits[j] = t;
        }
    cl_fmt_round(nums, nn, buf, sizeof(buf));
    printf("ближайшие следы к %s:\n", buf);
    if (k > nh) k = nh;
    for (hi = 0; hi < k; hi++) {
      cl_fmt_round(hits[hi].p, hits[hi].dim, buf, sizeof(buf));
      printf("  [%d] %s\n", hi, buf);
    }
    (void)hits_d;
    return 1;
  }

  printf("не распознал как команду: «%s»\n", line);
  printf("я world-model (dim=%d, steps=%llu), не свободный чат-бот.\n", mind->model.dim,
         (unsigned long long)mind->model.steps);
  printf("попробуй «что умеешь», «поговорим», «честно» или «помощь».\n");
  return 1;
}

static int run_chat_lite(const char *mind_path, const char *script) {
  const char *path = mind_path && mind_path[0] ? mind_path : "minds/agent.km";
  ProphetMem *mind;
  FILE *probe;
  pl_mems_reset();
  g_last_obs_n = 0;
  probe = fopen(path, "rb");
  if (probe) {
    fclose(probe);
    printf("loading %s\n", path);
    mind = pl_get(pl_load_mind(path));
  } else {
    printf("new mind → will save to %s\n", path);
    mind = pl_get(pl_mem_new(0.05, 128, 48));
  }
  printf("Kenga chat — живой world-model (lite, без Rust)\n");
  printf("разговор: «привет», «ты кто», «что умеешь», «поговорим», «честно»\n");
  printf("модель: «смотри 5 1 6», «что будет через 4», «обучи», «статус»\n\n");
  if (script) {
    char *buf = (char *)malloc(strlen(script) + 1);
    char *p, *line;
    if (!buf) die("oom");
    memcpy(buf, script, strlen(script) + 1);
    p = buf;
    while (*p) {
      line = p;
      while (*p && *p != '\n' && *p != '\r') p++;
      if (*p) {
        *p = 0;
        p++;
        if (*p == '\n') p++;
      }
      while (*line == ' ' || *line == '\t') line++;
      if (!*line || line[0] == '#') continue;
      printf("> %s\n", line);
      if (!cl_handle(&mind, path, line)) break;
    }
    free(buf);
    pl_mems_reset();
    return 0;
  }
  for (;;) {
    char line[1024];
    printf("kenga> ");
    fflush(stdout);
    if (!fgets(line, (int)sizeof(line), stdin)) break;
    {
      size_t n = strlen(line);
      while (n && (line[n - 1] == '\n' || line[n - 1] == '\r')) line[--n] = 0;
    }
    if (!line[0]) continue;
    if (!cl_handle(&mind, path, line)) break;
  }
  pl_mems_reset();
  return 0;
}

#endif /* KENGA_CHAT_LITE_INC */
