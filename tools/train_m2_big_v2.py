"""
M2 big v2: 2-layer softmax MLP with token embeddings.

Embedding dim = 16, hidden = 32, output = 28.
Params = 28*16 + 16*32 + 32 + 32*28 + 28 = 1916.

Trained on the same 168-file corpus; held-out = kenga_seed_*.
"""
import os
import sys
import numpy as np

KEYWORDS = {'fn','return','let','if','else','while','for','i64','println'}
TWO_CHAR = {'->','==','<=','>=','!=','&&','||','<<','>>','&','|','^','~'}

TOKENS = [
    'fn',     # 0
    'return', # 1
    'let',    # 2
    'if',     # 3
    'else',   # 4
    'while',  # 5
    'for',    # 6
    'i64',    # 7
    ':',      # 8
    ',',      # 9
    ';',      # 10
    '{',      # 11
    '}',      # 12
    '(',      # 13
    ')',      # 14
    '->',     # 15
    '+',      # 16
    '-',      # 17
    '*',      # 18
    '/',      # 19
    '=',      # 20
    '==',     # 21
    '<',      # 22
    '<=',     # 23
    '>',      # 24
    'println', # 25
    'ID',     # 26
    'NUM',    # 27
]
VOCAB = {tok: i for i, tok in enumerate(TOKENS)}
NUM_TOK = len(TOKENS)


def tokenize(src):
    out = []
    i = 0
    n = len(src)
    while i < n:
        c = src[i]
        if c == ' ' or c == '\t' or c == '\n' or c == '\r':
            i += 1
            continue
        if c == '/' and i+1 < n and src[i+1] == '/':
            while i < n and src[i] != '\n':
                i += 1
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
            while j < n and src[j].isdigit():
                j += 1
            out.append(VOCAB['NUM']); i = j; continue
        if c.isalpha() or c == '_':
            j = i
            while j < n and (src[j].isalnum() or src[j] == '_'):
                j += 1
            word = src[i:j]
            if word in KEYWORDS:
                out.append(VOCAB[word])
            else:
                out.append(VOCAB['ID'])
            i = j; continue
        i += 1
    return out


def collect_corpus():
    parts = []
    SKIP_BIG = {'bc_src_c.kenga','more.kenga','lower_kv.kenga','lower_c.kenga','rt_prophet.kenga','native_ml.kenga'}
    for root in ('kenga','examples'):
        for r,ds,fs in os.walk(root):
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
    train_files = [p for k,p in parts if k=='train']
    held_files = [p for k,p in parts if k=='held']
    print(f'corpus: {len(train_files)} train, {len(held_files)} held-out')

    big = []
    for p in train_files:
        try:
            tok = tokenize(open(p, encoding='utf-8', errors='replace').read())
            big.extend(tok)
        except Exception: pass
    print(f'total train tokens: {len(big)}')

    os.makedirs('minds', exist_ok=True)
    with open('minds/mid_prophet_m2_big_vocab.txt','w') as f:
        f.write(f'# vocab tokens = {NUM_TOK}\n')
        for tok, idx in VOCAB.items():
            f.write(f'{idx}\t{tok}\n')

    # Architecture: embed(K=8) -> sum+linear -> hidden(32) -> output(28)
    K = 8
    DIM = 32
    H = 64
    V = NUM_TOK
    rng = np.random.RandomState(7)
    E = rng.randn(V, DIM) * 0.1
    W1 = rng.randn(DIM, H) * (1.0 / np.sqrt(DIM))
    b1 = np.zeros(H)
    W2 = rng.randn(H, V) * (1.0 / np.sqrt(H))
    # init b2 with log(p(class)) so we start at marginal distribution
    arr_all = np.array(big, dtype=np.int32)
    counts = np.bincount(arr_all, minlength=V).astype(np.float32)
    b2 = np.log((counts + 1.0) / len(big))
    print(f'training: K={K} DIM={DIM} H={H} V={V} tot_params={V*DIM + DIM*H + H + H*V + V}')

    # Test: init accuracy = most-common-class rate
    init_logits = (W2 * 0) + b2  # zero weights to W1 acting on zero pool
    init_acc = 0
    print(f'init logits sanity: bias range = [{b2.min():.2f}, {b2.max():.2f}]')

    epochs = 60
    LR = 0.005
    b1_, b2_, eps = 0.9, 0.999, 1e-8
    def adam(m, v, d):
        m_ = b1_*m + (1-b1_)*d
        v_ = b2_*v + (1-b2_)*(d*d)
        mh = m_ / (1 - b1_**(ep+1))
        vh = v_ / (1 - b2_**(ep+1))
        return m_, v_, LR * mh / (np.sqrt(vh) + eps)

    n_total = len(big)
    train_arr = np.array(big, dtype=np.int32)
    mE = np.zeros_like(E); vE = np.zeros_like(E)
    mW1 = np.zeros_like(W1); vW1 = np.zeros_like(W1)
    mb1 = np.zeros_like(b1); vb1 = np.zeros_like(b1)
    mW2 = np.zeros_like(W2); vW2 = np.zeros_like(W2)
    mb2 = np.zeros_like(b2); vb2 = np.zeros_like(b2)

    for ep in range(epochs):
        # build all windows
        idx = np.arange(K, n_total)
        wins = np.stack([train_arr[idx-K+j] for j in range(K)], axis=1)  # (n-K, K)
        # embed: gather rows from E
        emb = E[wins]  # (n-K, K, DIM)
        # mean pool over K (axis 1)
        pooled = emb.mean(axis=1)  # (n-K, DIM)
        # hidden
        h_pre = pooled @ W1 + b1  # (n-K, H)
        h_act = np.tanh(h_pre)
        # logits
        logits = h_act @ W2 + b2  # (n-K, V)
        # softmax
        m = logits.max(axis=1, keepdims=True)
        exps = np.exp(logits - m)
        probs = exps / exps.sum(axis=1, keepdims=True)
        targets = train_arr[idx]
        # gradient
        g = probs.copy()
        g[np.arange(g.shape[0]), targets] -= 1.0
        g = g / g.shape[0]
        # backward
        dW2 = h_act.T @ g
        db2 = g.sum(axis=0)
        dh = g @ W2.T
        # tanh'
        dh = dh * (1 - h_act * h_act)
        dW1 = pooled.T @ dh
        db1 = dh.sum(axis=0)
        demb = dh @ W1.T  # (n-K, DIM)
        # mean pool grad
        demb_per_tok = np.zeros_like(emb)
        for j in range(K):
            demb_per_tok[:, j, :] = demb / K  # broadcast
        # gather gradients to E
        dE = np.zeros_like(E)
        np.add.at(dE, wins.reshape(-1), demb_per_tok.reshape(-1, DIM))
        # Adam
        mE, vE, upd = adam(mE, vE, dE); E -= upd
        mW1, vW1, upd = adam(mW1, vW1, dW1); W1 -= upd
        mb1, vb1, upd = adam(mb1, vb1, db1); b1 -= upd
        mW2, vW2, upd = adam(mW2, vW2, dW2); W2 -= upd
        mb2, vb2, upd = adam(mb2, vb2, db2); b2 -= upd
        # accuracy
        preds = probs.argmax(axis=1)
        acc = (preds == targets).mean() * 100
        print(f'  epoch {ep:>2d}: in-dist acc = {acc:.2f}%')

    # Save: write E,W1,W2,b1,b2 to single file
    SCALE = 1000
    with open('minds/mid_prophet_m2_big_w.txt','w') as f:
        f.write(f'vocab={V} k={K} dim={DIM} h={H} scale={SCALE} arch=mlp\n')
        f.write(f'[E] ' + ','.join(str(int(round(float(E[v,d])*SCALE))) for v in range(V) for d in range(DIM)) + '\n')
        f.write(f'[W1] ' + ','.join(str(int(round(float(W1[i,j])*SCALE))) for i in range(DIM) for j in range(H)) + '\n')
        f.write(f'[b1] ' + ','.join(str(int(round(float(b1[j])*SCALE))) for j in range(H)) + '\n')
        f.write(f'[W2] ' + ','.join(str(int(round(float(W2[i,j])*SCALE))) for i in range(H) for j in range(V)) + '\n')
        f.write(f'[b2] ' + ','.join(str(int(round(float(b2[j])*SCALE))) for j in range(V)) + '\n')

    # Held-out eval
    print(f'\n=== held-out evaluation ===')
    val_total = 0
    correct = 0
    for p in held_files:
        try:
            tok = tokenize(open(p, encoding='utf-8', errors='replace').read())
        except Exception:
            continue
        if len(tok) < K+1: continue
        arr = np.array(tok, dtype=np.int32)
        idx = np.arange(K, len(tok))
        wins = np.stack([arr[idx-K+j] for j in range(K)], axis=1)
        emb = E[wins]
        pooled = emb.mean(axis=1)
        h_pre = pooled @ W1 + b1
        h_act = np.tanh(h_pre)
        logits = h_act @ W2 + b2
        preds = logits.argmax(axis=1)
        c = int((preds == arr[idx]).sum())
        t = int(idx.shape[0])
        val_total += t
        correct += c
        name = p.replace('\\\\','/').split('/')[-1]
        print(f'  held {name}: {c}/{t} = {c*100/t:.2f}%')
    print(f'\noverall held-out: {correct}/{val_total} = {correct*100/val_total:.2f}%')

    # Also eval on last 10% of training as in-dist
    n = len(big); split = int(n*0.9)
    val = big[split:]; train_stream = big[:split]
    arr = np.array(val, dtype=np.int32) if len(val) > K else None
    if arr is not None:
        idx = np.arange(K, len(arr))
        wins = np.stack([arr[idx-K+j] for j in range(K)], axis=1)
        emb = E[wins]
        pooled = emb.mean(axis=1)
        h = np.tanh(pooled @ W1 + b1)
        logits = h @ W2 + b2
        c = int((logits.argmax(axis=1) == arr[idx]).sum())
        t = idx.shape[0]
        print(f'val_last10: {c}/{t} = {c*100/t:.2f}%')

    # save stream splits
    with open('minds/mid_prophet_m2_big_train.txt','w') as f:
        f.write(' '.join(str(x) for x in train_stream))
    with open('minds/mid_prophet_m2_big_test.txt','w') as f:
        f.write(' '.join(str(x) for x in val))


if __name__ == '__main__':
    main()
