"""
Mid-Prophet M3: real transformer-style decoder (numpy-only).

Configurable: width (D), depth (L blocks), context (K), heads (H).
Same training procedure as M3.0 regardless of size, so capacity is
the only axis changed.

Architecture:
  - 28-token vocabulary
  - K-token context window
  - learned embedding (28 -> D) + positional embedding (K -> D)
  - L decoder blocks: causal multi-head attention (H heads, head_dim=D//H)
                       with real QKV projections + Wo,
                       tanh FFN (D -> 2D -> D), residuals
  - softmax projection (D -> 28)
  - proper backprop through attention + FFN + embeddings

Train: mini-batch Adam over the full Kenga corpus. Held out: 9
kenga_seed_*.kenga programs (next-token accuracy).

Run:
  /c/Python314/python tools/train_m3.py
"""
import os
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


def collect_corpus():
    parts = []
    SKIP_BIG = {
        'bc_src_c.kenga','more.kenga','lower_kv.kenga','lower_c.kenga',
        'rt_prophet.kenga','native_ml.kenga','rt_vm.kenga','rt_tensor.kenga',
        'rt_kval_tape.kenga','rt_kval_mem.kenga',
    }
    for root in ('kenga', 'examples'):
        for r, ds, fs in os.walk(root):
            for f in fs:
                if not f.endswith('.kenga'): continue
                if f in SKIP_BIG: continue
                if f.startswith('mid_prophet') or f.startswith('pico_birth'): continue
                p = os.path.join(r, f)
                if 'kenga_seed_' in p:
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
        last_y = cur[:, -1, :]
        logits = last_y @ self.Wout + self.bout
        return logits, (x, X, caches, last_y)


class AdamOpt:
    def __init__(self, params, lr=0.005, b1=0.9, b2=0.999, eps=1e-8):
        self.lr = lr
        self.b1, self.b2, self.eps = b1, b2, eps
        self.m = {k: np.zeros_like(v) for k, v in params.items()}
        self.v = {k: np.zeros_like(v) for k, v in params.items()}
        self.t = 0
        self.params = params

    def step(self, grads):
        self.t += 1
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
    """Proper backprop through all blocks + embeddings."""
    x, X, caches, last_y = cache
    B, K, D, H, HEAD = x.shape[0], m.K, m.D, m.H, m.HEAD
    grads = {}

    logits = last_y @ m.Wout + m.bout
    probs = np.exp(logits - logits.max(axis=-1, keepdims=True))
    probs = probs / probs.sum(axis=-1, keepdims=True)
    g = probs.copy()
    g[np.arange(B), targets] -= 1.0
    g = g / B

    grads['Wout'] = last_y.T @ g
    grads['bout'] = g.sum(axis=0)
    dlast_y = g @ m.Wout.T

    dY = np.zeros((B, K, D))
    dY[:, -1, :] = dlast_y

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

    big = []
    for p in train_files:
        try:
            src = open(p, encoding='utf-8', errors='replace').read()
            big.extend(tokenize(src))
        except Exception:
            pass
    print(f'total train tokens: {len(big)}')

    os.makedirs('minds', exist_ok=True)
    with open('minds/mid_prophet_m3_vocab.txt', 'w') as f:
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

    rng = np.random.RandomState(11)
    m = M3(V, K, D, H, L, rng)
    m.bout = np.log((np.bincount(np.array(big, dtype=np.int32), minlength=V) + 1.0) / len(big))

    print(f'arch: K={K} D={D} H={H} L={L} V={V} params~{m.n_params()}')

    arr = np.array(big, dtype=np.int32)
    n = len(arr)

    params = m.params_map()
    opt = AdamOpt(params, lr=LR)

    for step in range(STEPS):
        starts = rng.randint(0, n - K, size=BATCH)
        xs = np.stack([arr[s:s + K] for s in starts])
        targets = arr[np.array(starts) + K]

        logits, cache = m.forward(xs)
        grads = backward(m, cache, targets)
        opt.step(grads)

        if step % EVAL_EVERY == 0 or step == STEPS - 1:
            preds = logits.argmax(axis=-1)
            acc = (preds == targets).mean() * 100
            print(f'  step {step:>4d}: batch-train-acc = {acc:.2f}%')

    # Save weights
    SCALE = 1000
    def dump(name, arr_):
        flat = arr_.reshape(-1)
        return f'[{name}] shape={list(arr_.shape)} ' + ','.join(str(int(round(float(x) * SCALE))) for x in flat) + '\n'

    w_path = f'minds/mid_prophet_{TAG}_w.txt'
    with open(w_path, 'w') as f:
        f.write(f'vocab={V} k={K} d={D} h={H} head={m.HEAD} layers={L} scale={SCALE} arch=transformer\n')
        for name, arr_ in params.items():
            f.write(dump(name, arr_))

    # Held-out
    print('\n=== held-out ===')
    val_total = 0
    correct_total = 0
    for p in held_files:
        try:
            tok = tokenize(open(p, encoding='utf-8', errors='replace').read())
        except Exception:
            continue
        if len(tok) < K + 1: continue
        arr_h = np.array(tok, dtype=np.int32)
        idx = np.arange(K, len(arr_h))
        wins = np.stack([arr_h[idx - K + j] for j in range(K)], axis=1)
        B = wins.shape[0]
        logits, _ = m.forward(wins)
        preds = logits.argmax(axis=-1)
        c = int((preds == arr_h[idx]).sum())
        t = int(idx.shape[0])
        val_total += t
        correct_total += c
        name = p.replace('\\\\', '/').split('/')[-1]
        print(f'  held {name}: {c}/{t} = {c*100/t:.2f}%')
    print(f'\noverall: {correct_total}/{val_total} = {correct_total*100/val_total:.2f}%')

    with open(f'minds/mid_prophet_{TAG}_meta.txt', 'w') as f:
        f.write(f'V={V}\nK={K}\nD={D}\nH={H}\nHEAD={m.HEAD}\nL={L}\n')
        f.write(f'train_tokens={n}\nsteps={STEPS}\nLR={LR}\nbatch={BATCH}\n')


if __name__ == '__main__':
    main()