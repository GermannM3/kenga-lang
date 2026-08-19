"""
Mid-Prophet M3: real transformer-style decoder (numpy-only).

Architecture:
  - 28-token vocabulary
  - K=32 token context window
  - learned embedding (28 -> 32) + positional embedding (K -> 32)
  - one decoder block: causal multi-head attention (4 heads, head_dim=8)
                       with real QKV projections, + tanh FFN (32 -> 64 -> 32),
                       residual connections
  - softmax projection (32 -> 28)
  - trained with proper backprop through attention + FFN + embeddings

Train: mini-batch SGD/Adam over the full Kenga corpus. Held out: 9
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


class M3:
    def __init__(self, V, K, D, H, rng):
        self.V = V
        self.K = K
        self.D = D
        self.H = H
        self.HEAD = D // H
        self.E_tok = rng.randn(V, D) * 0.10
        self.E_pos = rng.randn(K, D) * 0.10
        self.Wq = rng.randn(D, D) * 0.05
        self.Wk = rng.randn(D, D) * 0.05
        self.Wv = rng.randn(D, D) * 0.05
        self.Wo = rng.randn(D, D) * 0.05
        self.W1 = rng.randn(D, D * 2) * 0.04
        self.b1 = np.zeros(D * 2)
        self.W2 = rng.randn(D * 2, D) * 0.04
        self.b2 = np.zeros(D)
        self.Wout = rng.randn(D, V) * 0.05
        self.bout = None
        self.mask = np.triu(np.ones((K, K), dtype=bool), k=1)

    def forward(self, x):
        """x: (B, K) token ids -> logits (B, V). Returns logits + cached values."""
        B = x.shape[0]
        K, D, H, HEAD = self.K, self.D, self.H, self.HEAD
        E_tok, E_pos = self.E_tok, self.E_pos
        X = E_tok[x] + E_pos[np.arange(K)]  # (B, K, D)
        Q = X @ self.Wq
        K_ = X @ self.Wk
        V_ = X @ self.Wv
        q = Q.reshape(B, K, H, HEAD).transpose(0, 2, 1, 3)   # (B,H,K,HEAD)
        k = K_.reshape(B, K, H, HEAD).transpose(0, 2, 1, 3)
        v = V_.reshape(B, K, H, HEAD).transpose(0, 2, 1, 3)
        scores = q @ k.transpose(0, 1, 3, 2) / np.sqrt(HEAD)  # (B,H,K,K)
        scores = scores + np.where(self.mask, -1e9, 0.0)
        attn = np.exp(scores - scores.max(axis=-1, keepdims=True))
        attn = attn / attn.sum(axis=-1, keepdims=True)
        ctx = attn @ v  # (B,H,K,HEAD)
        ctx = ctx.transpose(0, 2, 1, 3).reshape(B, K, D)
        attn_out = X + ctx @ self.Wo  # residual 1

        h1 = attn_out @ self.W1 + self.b1  # (B,K,2D)
        act = np.tanh(h1)
        h2 = act @ self.W2 + self.b2       # (B,K,D)
        Y = attn_out + h2                  # residual 2

        last_y = Y[:, -1, :]
        logits = last_y @ self.Wout + self.bout
        return logits, (x, X, Q, K_, V_, q, k, v, scores, attn, ctx, attn_out, h1, act, h2, Y, last_y)


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


def backward(m, cache, targets):
    """Proper backprop through attention + FFN + embeddings.
    Returns dict of gradients for all params."""
    x, X, Q, K_, V_, q, k, v, scores, attn, ctx, attn_out, h1, act, h2, Y, last_y = cache
    B, K, D, H, HEAD = x.shape[0], m.K, m.D, m.H, m.HEAD
    grads = {}

    # --- head: softmax CE over logits ---
    logits = last_y @ m.Wout + m.bout
    probs = np.exp(logits - logits.max(axis=-1, keepdims=True))
    probs = probs / probs.sum(axis=-1, keepdims=True)
    g = probs.copy()
    g[np.arange(B), targets] -= 1.0
    g = g / B

    grads['Wout'] = last_y.T @ g
    grads['bout'] = g.sum(axis=0)
    dlast_y = g @ m.Wout.T  # (B, D)

    # --- residual 2: Y = attn_out + h2 ---
    dY = np.zeros_like(Y)
    dY[:, -1, :] = dlast_y
    dattn_out = dY.copy()
    dh2 = dY.copy()

    # FFN backward: h2 = act @ W2 + b2 ; act = tanh(h1) ; h1 = attn_out @ W1 + b1
    grads['W2'] = act.reshape(-1, D * 2).T @ dh2.reshape(-1, D)
    grads['b2'] = dh2.reshape(-1, D).sum(axis=0)
    dact = dh2 @ m.W2.T
    dh1 = dact * (1 - act ** 2)
    grads['W1'] = attn_out.reshape(-1, D).T @ dh1.reshape(-1, D * 2)
    grads['b1'] = dh1.reshape(-1, D * 2).sum(axis=0)
    dattn_out = dattn_out + dh1 @ m.W1.T  # from FFN input path

    # attention output: attn_out = X + ctx @ Wo
    dctx = dattn_out @ m.Wo.T  # (B,K,D)
    dX = dattn_out.copy()      # residual 1
    dWo = ctx.reshape(B * K, D).T @ dattn_out.reshape(B * K, D)
    grads['Wo'] = dWo

    # ctx heads: (B,H,K,HEAD)
    dctx = dctx.reshape(B, K, H, HEAD).transpose(0, 2, 1, 3)
    dv = attn.transpose(0, 1, 3, 2) @ dctx        # attn^T @ dctx
    dattn = dctx @ v.transpose(0, 1, 3, 2)         # dctx @ v^T

    # softmax backward
    dscores = attn * (dattn - (dattn * attn).sum(axis=-1, keepdims=True))
    dscores = np.where(m.mask, 0.0, dscores)      # masked positions have zero grad
    dscores = dscores / np.sqrt(HEAD)

    dq = dscores @ k                                # (B,H,K,HEAD)
    dk = dscores.transpose(0, 1, 3, 2) @ q
    dq = dq.transpose(0, 2, 1, 3).reshape(B, K, D)
    dk = dk.transpose(0, 2, 1, 3).reshape(B, K, D)
    dv = dv.transpose(0, 2, 1, 3).reshape(B, K, D)

    # QKV projections: Q = X @ Wq etc.
    grads['Wq'] = X.reshape(B * K, D).T @ dq.reshape(B * K, D)
    grads['Wk'] = X.reshape(B * K, D).T @ dk.reshape(B * K, D)
    grads['Wv'] = X.reshape(B * K, D).T @ dv.reshape(B * K, D)
    dX = dX + dq @ m.Wq.T + dk @ m.Wk.T + dv @ m.Wv.T

    # embeddings: X = E_tok[x] + E_pos[pos]
    grads['E_tok'] = np.zeros_like(m.E_tok)
    np.add.at(grads['E_tok'], x, dX)
    grads['E_pos'] = dX.sum(axis=0)

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

    K = 32
    D = 32
    H = 4
    rng = np.random.RandomState(11)
    m = M3(V, K, D, H, rng)
    m.bout = np.log((np.bincount(np.array(big, dtype=np.int32), minlength=V) + 1.0) / len(big))

    n_params = V * D + K * D + 4 * D * D + D * 2 * D + 2 * D + D * 2 * D + D + D * V + V
    print(f'arch: K={K} D={D} H={H} V={V} params~{n_params}')

    arr = np.array(big, dtype=np.int32)
    n = len(arr)

    params = {name: getattr(m, name) for name in
              ['E_tok','E_pos','Wq','Wk','Wv','Wo','W1','b1','W2','b2','Wout','bout']}
    opt = AdamOpt(params, lr=0.005)

    # Mini-batch windows: sample random start positions, windows of length K.
    BATCH = 256
    STEPS = 2400
    EVAL_EVERY = 400

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

    with open('minds/mid_prophet_m3_w.txt', 'w') as f:
        f.write(f'vocab={V} k={K} d={D} h={H} head={m.HEAD} scale={SCALE} arch=transformer\n')
        f.write(dump('E_tok', m.E_tok))
        f.write(dump('E_pos', m.E_pos))
        f.write(dump('Wq', m.Wq))
        f.write(dump('Wk', m.Wk))
        f.write(dump('Wv', m.Wv))
        f.write(dump('Wo', m.Wo))
        f.write(dump('W1', m.W1))
        f.write(dump('b1', m.b1))
        f.write(dump('W2', m.W2))
        f.write(dump('b2', m.b2))
        f.write(dump('Wout', m.Wout))
        f.write(dump('bout', m.bout))

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

    with open('minds/mid_prophet_m3_meta.txt', 'w') as f:
        f.write(f'V={V}\nK={K}\nD={D}\nH={H}\nHEAD={m.HEAD}\n')
        f.write(f'train_tokens={n}\nsteps={STEPS}\nLR={opt.lr}\n')


if __name__ == '__main__':
    main()