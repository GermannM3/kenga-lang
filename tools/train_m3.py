"""
Mid-Prophet M3: real transformer-style decoder (numpy-only).

Configurable: width (D), depth (L blocks), context (K), heads (H).
Same training procedure as M3.0 regardless of size, so capacity is
the only axis changed.

Architecture:
  - vocabulary: 28-token (M3.0/M3.1) OR 64-token learned codec (M3.2)
  - K-token context window
  - learned embedding (V -> D) + positional embedding (K -> D)
  - L decoder blocks: causal multi-head attention (H heads, head_dim=D//H)
                       with real QKV projections + Wo,
                       tanh FFN (D -> 2D -> D), residuals
  - softmax projection (D -> V)
  - proper backprop through attention + FFN + embeddings

Train: mini-batch Adam over the full Kenga corpus. Held out: 9
kenga_seed_*.kenga programs (next-token accuracy).

M3.2 ("representation expansion"): set M3_CODEC=1 to use the 64-token
codec (tools/codec_bpe.py, minds/kenga_bpe.pkl). Architecture, procedure,
dataset and eval are identical to M3.1; only the vocabulary changes.

Run:
  /c/Python314/python tools/train_m3.py
"""
import os
import subprocess
import numpy as np

KEYWORDS = {'fn','return','let','if','else','while','for','i64','println'}
TWO_CHAR = {'->','==','<=','>=','!=','&&','||','<<','>>','&','|','^','~'}

TOKENS = [
    'fn',     'return', 'let',    'if',     'else',   'while',
    'for',    'i64',    ':',      ',',      ';',      '{',
    '}',      '(',      ')',      '->',     '+',      '-',
    '*',      '/',      '=',      '==',     '<',      '<=',
    '>',      'println','ID',     'NUM',
]
V = len(TOKENS)
VOCAB = {t: i for i, t in enumerate(TOKENS)}


def tokenize(src):
    out = []
    i = 0
    n = len(src)
    while i < n:
        c = src[i]
        if c in ' \t\n\r':
            i += 1; continue
        if c == '/' and i+1 < n and src[i+1] == '/':
            while i < n and src[i] != '\n': i += 1
            continue
        two = src[i:i+2]
        if two in TWO_CHAR:
            out.append(VOCAB.get(two, VOCAB['ID'])); i += 2; continue
        if c in (':', ',', ';', '{', '}', '(', ')', '+', '-', '*', '/', '=', '<', '>'):
            if c == '-' and i+1 < n and src[i+1] == '>':
                out.append(VOCAB['->']); i += 2; continue
            out.append(VOCAB[c]); i += 1; continue
        if c.isdigit():
            j = i
            while j < n and src[j].isdigit(): j += 1
            out.append(VOCAB['NUM']); i = j; continue
        if c.isalpha() or c == '_':
            j = i
            while j < n and (src[j].isalnum() or src[j] == '_'):
                j += 1
            word = src[i:j]
            out.append(VOCAB[word] if word in KEYWORDS else VOCAB['ID'])
            i = j; continue
        i += 1
    return out


def make_codec():
    """Load the codec (minds/kenga_bpe.pkl or kenga_digits.pkl). Returns a
    Codec-like object exposing tokens, token_to_id, encode_word."""
    import pickle
    path = os.environ.get('M3_CODEC_FILE', 'minds/kenga_bpe.pkl')
    with open(path, 'rb') as f:
        data = pickle.load(f)
    tokens = data['tokens']
    merges = data['merges']
    id_to_token = tokens
    token_to_id = {t: i for i, t in enumerate(tokens)}
    merge_set = set(a + b for a, b in merges)
    keywords = KEYWORDS | ({'FIX'} if 'FIX' in token_to_id else set())

    def encode_word(w):
        if w in keywords:
            return [token_to_id[w]]
        spellable = all(ch in token_to_id for ch in w)
        if not spellable:
            return [token_to_id['ID']]
        toks = list(w)
        changed = True
        while changed:
            changed = False
            i = 0
            while i < len(toks) - 1:
                merged = toks[i] + toks[i+1]
                if merged in merge_set:
                    toks[i:i+2] = [merged]
                    changed = True
                i += 1
        return [token_to_id[t] for t in toks]

    return {
        'tokens': tokens,
        'id_to_token': id_to_token,
        'token_to_id': token_to_id,
        'encode_word': encode_word,
    }


def make_codec_tokenize(codec):
    """Tokenizer that uses the codec for identifiers, syntax as usual.
    M3_KEEP_COMMENTS=1 keeps // comment text as word tokens (NL->code)."""
    token_to_id = codec['token_to_id']
    encode_word = codec['encode_word']
    keep_comments = os.environ.get('M3_KEEP_COMMENTS', '0') == '1'

    def tokenize_codec(src):
        out = []
        i = 0
        n = len(src)
        while i < n:
            c = src[i]
            if c in ' \t\n\r':
                i += 1; continue
            if c == '/' and i+1 < n and src[i+1] == '/':
                if not keep_comments:
                    while i < n and src[i] != '\n': i += 1
                    continue
                i += 2
                j = i
                while j < n and src[j] != '\n':
                    if src[j].isalnum() or src[j] == '_':
                        e = j
                        while e < n and (src[e].isalnum() or src[e] == '_'):
                            e += 1
                        out.extend(encode_word(src[j:e]))
                        j = e
                    else:
                        j += 1
                i = j
                continue
            two = src[i:i+2]
            if two in TWO_CHAR:
                out.append(token_to_id.get(two, token_to_id['ID'])); i += 2; continue
            if c in (':', ',', ';', '{', '}', '(', ')', '+', '-', '*', '/', '=', '<', '>'):
                if c == '-' and i+1 < n and src[i+1] == '>':
                    out.append(token_to_id['->']); i += 2; continue
                out.append(token_to_id[c]); i += 1; continue
            if c.isdigit():
                j = i
                while j < n and src[j].isdigit(): j += 1
                if 'NUM' in token_to_id:
                    out.append(token_to_id['NUM']); i = j; continue
                # digit codec: emit each digit as its own token
                for d in src[i:j]:
                    out.append(token_to_id.get(d, token_to_id['ID']))
                i = j; continue
            if c.isalpha() or c == '_':
                j = i
                while j < n and (src[j].isalnum() or src[j] == '_'):
                    j += 1
                word = src[i:j]
                out.extend(encode_word(word))
                i = j; continue
            i += 1
        return out
    return tokenize_codec


def collect_corpus():
    extra = os.environ.get('M3_EXTRA_DIR')
    if os.environ.get('M3_ONLY_EXTRA', '0') == '1' and extra:
        parts = []
        for f in sorted(os.listdir(extra)):
            p = os.path.join(extra, f)
            try:
                open(p, encoding='utf-8', errors='replace').read()
                parts.append(('train', p))
            except Exception:
                pass
        return parts
    if os.environ.get('M3_ONLY_FACTORY', '0') == '1':
        return []
    parts = []
    SKIP_BIG = {
        'bc_src_c.kenga','more.kenga','lower_kv.kenga','lower_c.kenga',
        'rt_prophet.kenga','native_ml.kenga','rt_vm.kenga','rt_tensor.kenga',
        'rt_kval_tape.kenga','rt_kval_mem.kenga',
    }
    include_big = os.environ.get('M3_INCLUDE_BIG', '0') == '1'
    # Phase II M5.2: file-level holdout of REAL code (whole files, not rows)
    import hashlib
    real_split = os.environ.get('M3_REAL_SPLIT', '')
    holdout_frac = float(real_split) if real_split else 0.0
    for root in ('kenga', 'examples'):
        for r, ds, fs in os.walk(root):
            for f in fs:
                if not f.endswith('.kenga'): continue
                if not include_big and f in SKIP_BIG: continue
                if f.startswith('mid_prophet') or f.startswith('pico_birth'): continue
                p = os.path.join(r, f)
                if holdout_frac > 0:
                    h = int(hashlib.md5(p.replace('\\\\', '/').encode()).hexdigest(), 16) % 10000
                    side = 'held' if h < holdout_frac * 10000 else 'train'
                    parts.append((side, p))
                    continue
                if 'kenga_seed_' in p:
                    if os.environ.get('M3_INCLUDE_SEEDS') == '1':
                        repeat = int(os.environ.get('M3_SEED_REPEAT', '1'))
                        for _ in range(max(1, repeat)):
                            parts.append(('train', p))
                    else:
                        parts.append(('held', p))
                    continue
                try:
                    data = open(p, encoding='utf-8', errors='replace').read()
                    parts.append(('train', p))
                except Exception:
                    pass
    return parts


class Block:
    """One transformer decoder block: self-attention + tanh FFN, both residual."""
    def __init__(self, D, FF, rng):
        self.Wq = rng.randn(D, D) * 0.05
        self.Wk = rng.randn(D, D) * 0.05
        self.Wv = rng.randn(D, D) * 0.05
        self.Wo = rng.randn(D, D) * 0.05
        self.W1 = rng.randn(D, FF) * 0.04
        self.b1 = np.zeros(FF)
        self.W2 = rng.randn(FF, D) * 0.04
        self.b2 = np.zeros(D)

    def params(self):
        return ['Wq', 'Wk', 'Wv', 'Wo', 'W1', 'b1', 'W2', 'b2']


class M3:
    def __init__(self, V, K, D, H, L, rng):
        self.V, self.K, self.D, self.H, self.L = V, K, D, H, L
        self.HEAD = D // H
        self.FF = D * 2
        self.E_tok = rng.randn(V, D) * 0.10
        self.E_pos = rng.randn(K, D) * 0.10
        self.blocks = [Block(D, self.FF, rng) for _ in range(L)]
        self.Wout = rng.randn(D, V) * 0.05
        self.bout = None
        self.mask = np.triu(np.ones((K, K), dtype=bool), k=1)

    def n_params(self):
        n = self.V * self.D + self.K * self.D
        n += sum(4 * self.D * self.D + 2 * self.D * self.FF + self.FF + self.D
                 for _ in self.blocks)
        n += self.D * self.V + self.V
        return n

    def params_map(self):
        """Flatten all params into {name: array} for the optimizer."""
        p = {}
        p['E_tok'] = self.E_tok
        p['E_pos'] = self.E_pos
        for li, blk in enumerate(self.blocks):
            for name in blk.params():
                p[f'{li}:{name}'] = getattr(blk, name)
        p['Wout'] = self.Wout
        p['bout'] = self.bout
        return p

    def forward(self, x):
        """x: (B, K) token ids -> logits (B, V). Returns logits + caches."""
        B = x.shape[0]
        K, D, H, HEAD = self.K, self.D, self.H, self.HEAD
        X = self.E_tok[x] + self.E_pos[np.arange(K)]  # (B, K, D)
        cur = X
        caches = []  # per-block: (X_in, Q, K_, V_, q, k, v, scores, attn, ctx, attn_out, h1, act, h2, Y)
        for blk in self.blocks:
            Q = cur @ blk.Wq
            K_ = cur @ blk.Wk
            V_ = cur @ blk.Wv
            q = Q.reshape(B, K, H, HEAD).transpose(0, 2, 1, 3)
            k = K_.reshape(B, K, H, HEAD).transpose(0, 2, 1, 3)
            v = V_.reshape(B, K, H, HEAD).transpose(0, 2, 1, 3)
            scores = q @ k.transpose(0, 1, 3, 2) / np.sqrt(HEAD)
            scores = scores + np.where(self.mask, -1e9, 0.0)
            attn = np.exp(scores - scores.max(axis=-1, keepdims=True))
            attn = attn / attn.sum(axis=-1, keepdims=True)
            ctx = attn @ v
            ctx = ctx.transpose(0, 2, 1, 3).reshape(B, K, D)
            attn_out = cur + ctx @ blk.Wo
            h1 = attn_out @ blk.W1 + blk.b1
            act = np.tanh(h1)
            h2 = act @ blk.W2 + blk.b2
            Y = attn_out + h2
            caches.append((cur, Q, K_, V_, q, k, v, scores, attn, ctx, attn_out, h1, act, h2, Y))
            cur = Y
        logits = cur @ self.Wout + self.bout
        return logits, (x, X, caches, cur)


class AdamOpt:
    def __init__(self, params, lr=0.005, b1=0.9, b2=0.999, eps=1e-8):
        self.lr = lr
        self.b1, self.b2, self.eps = b1, b2, eps
        self.m = {k: np.zeros_like(v) for k, v in params.items()}
        self.v = {k: np.zeros_like(v) for k, v in params.items()}
        self.t = 0
        self.params = params

    def step(self, grads, max_norm=None):
        self.t += 1
        if max_norm:
            # global-norm gradient clipping (prevents loss explosions)
            total = 0.0
            for g in grads.values():
                total += float((g * g).sum())
            total = total ** 0.5
            if total > max_norm:
                scale = max_norm / (total + 1e-12)
                grads = {k: g * scale for k, g in grads.items()}
        for k, g in grads.items():
            self.m[k] = self.b1 * self.m[k] + (1 - self.b1) * g
            self.v[k] = self.b2 * self.v[k] + (1 - self.b2) * (g * g)
            m_hat = self.m[k] / (1 - self.b1 ** self.t)
            v_hat = self.v[k] / (1 - self.b2 ** self.t)
            self.params[k] -= self.lr * m_hat / (np.sqrt(v_hat) + self.eps)


def block_backward(blk, cache, dY, m, B, K, D, H, HEAD):
    """Backward through one block. dY: gradient on block output (B,K,D).
    Returns (grads_prefix, dXin) where grads_prefix keys are block-local
    ('Wq','Wk',...) and dXin is gradient on block input."""
    cur, Q, K_, V_, q, k, v, scores, attn, ctx, attn_out, h1, act, h2, Y = cache
    g = {}
    dattn_out = dY.copy()
    dh2 = dY.copy()
    g['W2'] = act.reshape(-1, D * 2).T @ dh2.reshape(-1, D)
    g['b2'] = dh2.reshape(-1, D).sum(axis=0)
    dact = dh2 @ blk.W2.T
    dh1 = dact * (1 - act ** 2)
    g['W1'] = attn_out.reshape(-1, D).T @ dh1.reshape(-1, D * 2)
    g['b1'] = dh1.reshape(-1, D * 2).sum(axis=0)
    dattn_out = dattn_out + dh1 @ blk.W1.T

    dctx = dattn_out @ blk.Wo.T
    dX = dattn_out.copy()  # residual 1
    g['Wo'] = ctx.reshape(B * K, D).T @ dattn_out.reshape(B * K, D)

    dctx = dctx.reshape(B, K, H, HEAD).transpose(0, 2, 1, 3)
    dv = attn.transpose(0, 1, 3, 2) @ dctx
    dattn = dctx @ v.transpose(0, 1, 3, 2)
    dscores = attn * (dattn - (dattn * attn).sum(axis=-1, keepdims=True))
    dscores = np.where(m.mask, 0.0, dscores)
    dscores = dscores / np.sqrt(HEAD)

    dq = dscores @ k
    dk = dscores.transpose(0, 1, 3, 2) @ q
    dq = dq.transpose(0, 2, 1, 3).reshape(B, K, D)
    dk = dk.transpose(0, 2, 1, 3).reshape(B, K, D)
    dv = dv.transpose(0, 2, 1, 3).reshape(B, K, D)

    g['Wq'] = cur.reshape(B * K, D).T @ dq.reshape(B * K, D)
    g['Wk'] = cur.reshape(B * K, D).T @ dk.reshape(B * K, D)
    g['Wv'] = cur.reshape(B * K, D).T @ dv.reshape(B * K, D)
    dX = dX + dq @ blk.Wq.T + dk @ blk.Wk.T + dv @ blk.Wv.T
    return g, dX


def backward(m, cache, targets):
    """Proper backprop through all blocks + embeddings.

    targets: (B, K) - for each window position j, the next token arr[s+j+1].
    Logits (B, K, V) predict each position's next token (causal LM objective).
    """
    x, X, caches, Y = cache
    B, K, D, H, HEAD = x.shape[0], m.K, m.D, m.H, m.HEAD
    V = m.V
    grads = {}

    logits = Y @ m.Wout + m.bout
    probs = np.exp(logits - logits.max(axis=-1, keepdims=True))
    probs = probs / probs.sum(axis=-1, keepdims=True)
    g = probs.copy()
    g[np.arange(B)[:, None], np.arange(K)[None, :], targets] -= 1.0
    g = g / (B * K)

    grads['Wout'] = Y.reshape(B * K, D).T @ g.reshape(B * K, V)
    grads['bout'] = g.reshape(B * K, V).sum(axis=0)
    dY = g @ m.Wout.T

    # Backward through blocks in reverse
    for li in range(m.L - 1, -1, -1):
        blk = m.blocks[li]
        g_blk, dY = block_backward(blk, caches[li], dY, m, B, K, D, H, HEAD)
        for name, arr in g_blk.items():
            grads[f'{li}:{name}'] = arr

    # embeddings: X = E_tok[x] + E_pos[pos]
    grads['E_tok'] = np.zeros_like(m.E_tok)
    np.add.at(grads['E_tok'], x, dY)
    grads['E_pos'] = dY.sum(axis=0)

    return grads


def main():
    parts = collect_corpus()
    train_files = [p for k, p in parts if k == 'train']
    held_files = [p for k, p in parts if k == 'held']
    print(f'corpus: {len(train_files)} train, {len(held_files)} held-out')

    # ---- vocabulary (M3_CODEC=1 switches to the 64-token learned codec) ----
    global V, VOCAB, TOKENS, tokenize
    use_codec = os.environ.get('M3_CODEC', '0') == '1'
    if use_codec:
        codec = make_codec()
        TOKENS = codec['tokens']
        V = len(TOKENS)
        VOCAB = codec['token_to_id']
        tokenize = make_codec_tokenize(codec)
        is_digits = 'NUM' not in VOCAB
        print(f'vocab: codec (len={V}, digits={is_digits})')
        vocab_path = 'minds/mid_prophet_m37_vocab.txt' if is_digits else 'minds/mid_prophet_m32_vocab.txt'
    else:
        V = len(TOKENS)
        print(f'vocab: 28-token (len={V})')
        vocab_path = 'minds/mid_prophet_m3_vocab.txt'

    big = []
    for p in train_files:
        try:
            src = open(p, encoding='utf-8', errors='replace').read()
            big.extend(tokenize(src))
        except Exception:
            pass
    # Corpus Factory (Phase II): verified synthetic programs. Primaries and
    # semantic-equivalence variants only; mutants are NOT LM training text.
    factory_path = os.environ.get('M3_FACTORY')
    n_factory = 0
    if factory_path:
        import json
        with open(factory_path, encoding='utf-8') as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue
                rec = json.loads(line)
                big.extend(tokenize(rec['src']))
                for v in rec.get('variants', []):
                    big.extend(tokenize(v['src']))
                n_factory += 1
        print(f'factory corpus: {factory_path} ({n_factory} programs)')
    print(f'total train tokens: {len(big)}')

    os.makedirs('minds', exist_ok=True)
    with open(vocab_path, 'w') as f:
        f.write(f'# vocab = {V}\n')
        for t, idx in VOCAB.items():
            f.write(f'{idx}\t{t}\n')

    # ---- config (capacity is the only thing that changes between runs) ----
    K = int(os.environ.get('M3_K', 64))
    D = int(os.environ.get('M3_D', 48))
    H = int(os.environ.get('M3_H', 6))
    L = int(os.environ.get('M3_L', 2))
    assert D % H == 0
    BATCH = int(os.environ.get('M3_BATCH', 128))
    STEPS = int(os.environ.get('M3_STEPS', 2000))
    EVAL_EVERY = int(os.environ.get('M3_EVAL_EVERY', 400))
    LR = float(os.environ.get('M3_LR', 0.005))
    TAG = os.environ.get('M3_TAG', 'm31')
    # ---------------------------------------------------------------------

    rng = np.random.RandomState(int(os.environ.get('M3_SEED', '11')))
    m = M3(V, K, D, H, L, rng)
    m.bout = np.log((np.bincount(np.array(big, dtype=np.int32), minlength=V) + 1.0) / len(big))

    print(f'arch: K={K} D={D} H={H} L={L} V={V} params~{m.n_params()}')

    arr = np.array(big, dtype=np.int32)
    n = len(arr)

    params = m.params_map()
    opt = AdamOpt(params, lr=LR)

    SCALE = 1000
    def dump(name, arr_):
        flat = arr_.reshape(-1)
        return f'[{name}] shape={list(arr_.shape)} ' + ','.join(str(int(round(float(x) * SCALE))) for x in flat) + '\n'

    def write_weights(path):
        with open(path, 'w') as f:
            f.write(f'vocab={V} k={K} d={D} h={H} head={m.HEAD} layers={L} scale={SCALE} arch=transformer\n')
            for name, arr_ in params.items():
                f.write(dump(name, arr_))

    for step in range(STEPS):
        starts = rng.randint(0, n - K, size=BATCH)
        xs = np.stack([arr[s:s + K] for s in starts])
        targets = np.stack([arr[s + 1:s + K + 1] for s in starts])

        logits, cache = m.forward(xs)
        grads = backward(m, cache, targets)
        opt.step(grads, max_norm=float(os.environ.get('M3_CLIP', '0') or 0) or None)

        if step % EVAL_EVERY == 0 or step == STEPS - 1:
            preds = logits.argmax(axis=-1)
            acc = (preds == targets).mean() * 100
            logp = np.log(np.exp(logits - logits.max(axis=-1, keepdims=True))
                          .sum(axis=-1, keepdims=True))
            nll = -(np.take_along_axis(
                logits - logits.max(axis=-1, keepdims=True)
                - logp, targets[..., None], axis=-1)).mean()
            print(f'  step {step:>4d}: batch-train-acc = {acc:.2f}%  '
                  f'loss = {float(nll):.4f}')
            # rolling snapshot: crash/reboot loses at most EVAL_EVERY steps
            write_weights(f'minds/mid_prophet_{TAG}_snap_w.txt')

    w_path = f'minds/mid_prophet_{TAG}_w.txt'
    write_weights(w_path)

    # Held-out
    print('\n=== held-out ===')
    held_docs = []
    for p in held_files:
        try:
            held_docs.append((os.path.basename(p),
                              open(p, encoding='utf-8', errors='replace').read()))
        except Exception:
            pass
    factory_holdout = os.environ.get('M3_FACTORY_HOLDOUT')
    if factory_holdout:
        import json
        parts_h = []
        with open(factory_holdout, encoding='utf-8') as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue
                rec = json.loads(line)
                parts_h.append(rec['src'])
        # short programs (<K tokens): concatenate into one stream so that
        # sliding-window evaluation still has full windows
        held_docs.append(('factory_test', '\n'.join(parts_h)))
        print(f'factory holdout: {factory_holdout} ({len(parts_h)} programs, combined stream)')
    val_total = 0
    correct_total = 0
    heldout_results = []
    for name, text in held_docs:
        tok = tokenize(text)
        if len(tok) < K + 1: continue
        arr_h = np.array(tok, dtype=np.int32)
        # non-overlapping windows + chunked forwards (memory-safe for big K/D/L)
        idx_all = np.arange(K, len(arr_h), K)
        c_sum = 0
        t_sum = 0
        for s0 in range(0, len(idx_all), 128):
            chunk = idx_all[s0:s0 + 128]
            wins = np.stack([arr_h[chunk - K + j] for j in range(K)], axis=1)
            logits, _ = m.forward(wins)
            preds = logits.argmax(axis=-1)
            targets = np.stack([arr_h[chunk - K + 1 + j] for j in range(K)], axis=1)
            c_sum += int((preds == targets).sum())
            t_sum += int(preds.size)
        val_total += t_sum
        correct_total += c_sum
        heldout_results.append((name, c_sum, t_sum))
        print(f'  held {name}: {c_sum}/{t_sum} = {c_sum*100/max(1,t_sum):.2f}%')
    if val_total:
        print(f'\noverall: {correct_total}/{val_total} = {correct_total*100/val_total:.2f}%')
    else:
        print('\nno held-out files (all seeds in training)')

    with open(f'minds/mid_prophet_{TAG}_meta.txt', 'w') as f:
        f.write(f'V={V}\nK={K}\nD={D}\nH={H}\nHEAD={m.HEAD}\nL={L}\n')
        f.write(f'train_tokens={n}\nsteps={STEPS}\nLR={LR}\nbatch={BATCH}\n')

    # ---- run manifest: reproducible experiment artifact -----------------
    import hashlib
    import json
    import time

    def sha256_file(path, _buf=1 << 20):
        h = hashlib.sha256()
        try:
            with open(path, 'rb') as fh:
                while True:
                    b = fh.read(_buf)
                    if not b:
                        break
                    h.update(b)
            return h.hexdigest()
        except OSError:
            return None

    try:
        git_commit = subprocess.check_output(
            ['git', 'rev-parse', 'HEAD'], stderr=subprocess.DEVNULL,
        ).decode().strip()
    except Exception:
        git_commit = None

    manifest = {
        'tag': TAG,
        'finished': time.strftime('%Y-%m-%d %H:%M:%S'),
        'git_commit': git_commit,
        'arch': {'V': V, 'K': K, 'D': D, 'H': H, 'L': L,
                 'params': m.n_params()},
        'train': {'tokens': n, 'steps': STEPS, 'lr': LR, 'batch': BATCH,
                  'clip': os.environ.get('M3_CLIP'),
                  'codec_file': os.environ.get('M3_CODEC_FILE'),
                  'codec_sha256': sha256_file(os.environ.get('M3_CODEC_FILE', '')),
                  'factory': os.environ.get('M3_FACTORY'),
                  'factory_sha256': sha256_file(os.environ.get('M3_FACTORY', '')),
                  'real_split': os.environ.get('M3_REAL_SPLIT'),
                  'include_big': os.environ.get('M3_INCLUDE_BIG'),
                  'extra_dir': os.environ.get('M3_EXTRA_DIR')},
        'heldout': {name: round(100 * c / max(1, t), 2)
                    for name, c, t in heldout_results},
        'weights': w_path,
        'weights_sha256': sha256_file(w_path),
        'snapshot': f'minds/mid_prophet_{TAG}_snap_w.txt',
    }
    with open(f'minds/mid_prophet_{TAG}_run.json', 'w', encoding='utf-8') as f:
        json.dump(manifest, f, indent=2)
    print(f'run manifest: minds/mid_prophet_{TAG}_run.json')


if __name__ == '__main__':
    main()