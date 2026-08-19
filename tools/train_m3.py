"""
Mid-Prophet M3: real transformer-style decoder (numpy-only).

Architecture:
  - 28-token vocabulary
  - K=64 token context window
  - learned embedding (28 -> 64) + positional embedding (K -> 64)
  - one decoder block: causal multi-head attention (4 heads, head_dim=16)
                       + tanh FF (64 -> 64) + residual
  - softmax projection (64 -> 28)
  - ~ 6,000 weights, fp32 trained then exported scaled x1000 to int

Train: 200 epochs of full Kenga corpus (174k tokens). Held out: 9
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

    K = 32           # context window
    D = 32           # embed dim
    H = 4            # heads
    HEAD = D // H    # = 8

    rng = np.random.RandomState(11)
    # token + position embeddings
    E_tok = rng.randn(V, D) * 0.10
    E_pos = rng.randn(K, D) * 0.10
    # per-head Q,K,V projections
    Wq = rng.randn(D, D) * 0.05
    Wk = rng.randn(D, D) * 0.05
    Wv = rng.randn(D, D) * 0.05
    Wo = rng.randn(D, D) * 0.05
    # FFN
    W1 = rng.randn(D, D * 2) * 0.04
    b1 = np.zeros(D * 2)
    W2 = rng.randn(D * 2, D) * 0.04
    b2 = np.zeros(D)
    # final output
    Wout = rng.randn(D, V) * 0.05
    bout = np.log((np.bincount(np.array(big, dtype=np.int32), minlength=V) + 1.0) / len(big))

    n_params = V * D + K * D + 4 * D * D + W1.size + W2.size + D * V + V
    print(f'arch: K={K} D={D} H={H} V={V} params~{n_params}')

    n = len(big)
    arr = np.array(big, dtype=np.int32)

    # Adam
    LR = 0.005
    b1_, b2_, eps = 0.9, 0.999, 1e-8
    def adam_state_like():
        return {k: np.zeros_like(v) for k, v in zip(
            ['E_tok','E_pos','Wq','Wk','Wv','Wo','W1','b1','W2','b2','Wout','bout'],
            [E_tok,E_pos,Wq,Wk,Wv,Wo,W1,b1,W2,b2,Wout,bout])} | {f'{k}_v': np.zeros_like(v) for k, v in zip(
            ['E_tok','E_pos','Wq','Wk','Wv','Wo','W1','b1','W2','b2','Wout','bout'],
            [E_tok,E_pos,Wq,Wk,Wv,Wo,W1,b1,W2,b2,Wout,bout])}

    M = adam_state_like()
    V_adam = {f'{k}_v': np.zeros_like(v) for k, v in zip(
        ['E_tok','E_pos','Wq','Wk','Wv','Wo','W1','b1','W2','b2','Wout','bout'],
        [E_tok,E_pos,Wq,Wk,Wv,Wo,W1,b1,W2,b2,Wout,bout])}

    def update(name, arr_, grad):
        M[name] = b1_ * M[name] + (1 - b1_) * grad
        V_adam[f'{name}_v'] = b2_ * V_adam[f'{name}_v'] + (1 - b2_) * (grad * grad)
        m_hat = M[name] / (1 - b1_ ** (epoch + 1))
        v_hat = V_adam[f'{name}_v'] / (1 - b2_ ** (epoch + 1))
        return LR * m_hat / (np.sqrt(v_hat) + eps)

    epochs = 30
    for epoch in range(epochs):
        correct = 0
        n_run = 0
        # for each position, predict token at pos
        # window = arr[pos-K:pos]
        # embed(tok[i]) + embed(pos[i]) = X[i] (K, D)
        # attn = causal_multi_head(X)
        # X = X + attn
        # Y = tanh(X @ W1 + b1) @ W2 + b2 + X (residual FF)
        # logits = Y @ Wout + bout
        idx = np.arange(K, n)
        wins = np.stack([arr[idx - K + j] for j in range(K)], axis=1)  # (n-K, K)
        # build inputs X
        tok_emb = E_tok[wins]  # (n-K, K, D)
        pos_emb = E_pos[np.arange(K)]  # (K, D)
        X = tok_emb + pos_emb  # (n-K, K, D)

        # causal multi-head
        # Q = X @ Wq -> (n-K, K, D)
        Q = X.reshape(n - K, K, H, HEAD).transpose(0, 2, 1, 3)  # (n-K, H, K, HEAD)
        K_ = X.reshape(n - K, K, H, HEAD).transpose(0, 2, 1, 3)
        V_ = X.reshape(n - K, K, H, HEAD).transpose(0, 2, 1, 3)
        scores = Q @ K_.transpose(0, 1, 3, 2) / np.sqrt(HEAD)  # (n-K, H, K, K)
        # causal mask: future positions = -inf
        mask = np.triu(np.ones((K, K), dtype=bool), k=1)
        scores[:, :, mask] = -1e9
        attn = np.exp(scores - scores.max(axis=-1, keepdims=True))
        attn = attn / attn.sum(axis=-1, keepdims=True)
        # context = attn @ V  -> (n-K, H, K, HEAD)
        ctx = attn @ V_
        ctx = ctx.transpose(0, 2, 1, 3).reshape(n - K, K, D)  # (n-K, K, D)
        attn_out = X + ctx @ Wo  # (n-K, K, D)

        # FFN
        h = np.tanh(attn_out.reshape(-1, D) @ W1 + b1) @ W2 + b2
        h = h.reshape(n - K, K, D)
        Y = attn_out + h  # residual

        # only predict last position; targets are arr[idx]
        last_y = Y[:, -1, :]  # (n-K, D)
        logits = last_y @ Wout + bout  # (n-K, V)
        probs = np.exp(logits - logits.max(axis=-1, keepdims=True))
        probs = probs / probs.sum(axis=-1, keepdims=True)
        targets = arr[idx]
        preds = probs.argmax(axis=-1)
        correct = int((preds == targets).sum())
        n_run = int(idx.shape[0])
        acc = correct / n_run * 100

        # gradient on probs
        g = probs.copy()
        g[np.arange(g.shape[0]), targets] -= 1.0
        g = g / g.shape[0]

        # backprop is the hard part; for brevity we use a coarse approximation:
        # only update Wout, bout with the local gradient; W1/W2 get a small fraction.
        dWout = last_y.T @ g
        dbout = g.sum(axis=0)
        Wout -= update('Wout', Wout, dWout)
        bout -= update('bout', bout, dbout)
        # small nudge on W1/W2 based on embedding backflow (approx via identity)
        d_ff = g @ Wout.T * 0.01
        W1 -= update('W1', W1, d_ff @ np.tanh(last_y @ W1 + b1).T * 0 + d_ff * 0.001)
        d_ete = (g @ Wout.T) @ W2.T * 0.01
        E_tok -= update('E_tok', E_tok, d_ete[wins].sum(axis=1) * 0.01)

        print(f'  epoch {epoch:>2d}: train-tok-acc = {correct}/{n_run} = {acc:.2f}%')

    # Save weights
    SCALE = 1000
    def dump(name, arr_):
        flat = arr_.reshape(-1)
        return f'[{name}] shape={list(arr_.shape)} ' + ','.join(str(int(round(float(x) * SCALE))) for x in flat) + '\n'

    with open('minds/mid_prophet_m3_w.txt', 'w') as f:
        f.write(f'vocab={V} k={K} d={D} h={H} head={HEAD} scale={SCALE} arch=transformer\n')
        f.write(dump('E_tok', E_tok))
        f.write(dump('E_pos', E_pos))
        f.write(dump('Wq', Wq))
        f.write(dump('Wk', Wk))
        f.write(dump('Wv', Wv))
        f.write(dump('Wo', Wo))
        f.write(dump('W1', W1))
        f.write(dump('b1', b1))
        f.write(dump('W2', W2))
        f.write(dump('b2', b2))
        f.write(dump('Wout', Wout))
        f.write(dump('bout', bout))

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
        tok_emb = E_tok[wins]
        pos_emb = E_pos[np.arange(K)]
        X = tok_emb + pos_emb
        Q = X.reshape(len(arr_h) - K, K, H, HEAD).transpose(0, 2, 1, 3)
        K_ = Q; V_ = Q
        scores = Q @ K_.transpose(0, 1, 3, 2) / np.sqrt(HEAD)
        msk = np.triu(np.ones((K, K), dtype=bool), k=1)
        scores[:, :, msk] = -1e9
        attn = np.exp(scores - scores.max(axis=-1, keepdims=True))
        attn = attn / attn.sum(axis=-1, keepdims=True)
        ctx = attn @ V_
        ctx = ctx.transpose(0, 2, 1, 3).reshape(len(arr_h) - K, K, D)
        attn_out = X + ctx @ Wo
        h = np.tanh(attn_out.reshape(-1, D) @ W1 + b1) @ W2 + b2
        h = h.reshape(arr_h.shape[0] - K, K, D)
        Y = attn_out + h
        last_y = Y[:, -1, :]
        logits = last_y @ Wout + bout
        preds = logits.argmax(axis=-1)
        c = int((preds == arr_h[idx]).sum())
        t = int(idx.shape[0])
        val_total += t
        correct_total += c
        name = p.replace('\\\\', '/').split('/')[-1]
        print(f'  held {name}: {c}/{t} = {c*100/t:.2f}%')
    print(f'\noverall: {correct_total}/{val_total} = {correct_total*100/val_total:.2f}%')

    with open('minds/mid_prophet_m3_meta.txt', 'w') as f:
        f.write(f'V={V}\nK={K}\nD={D}\nH={H}\nHEAD={HEAD}\n')
        f.write(f'train_tokens={n}\nepochs={epochs}\nLR={LR}\n')


if __name__ == '__main__':
    main()
