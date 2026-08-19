"""
Mid-Prophet M2: large-corpus variant.

Trains a token-level linear softmax predictor over the ~300k-token
kenga source corpus (~186 .kenga files). Uses numpy for tractable
training time.

Outputs:
  minds/mid_prophet_m2_k16_w.txt       -- integer weights, scale=1000
  minds/mid_prophet_m2_k16_vocab.txt   -- 28-token codec
  minds/mid_prophet_m2_k16_train.txt   -- first 90% (token ids)
  minds/mid_prophet_m2_k16_test.txt    -- last 10% (token ids)
  minds/mid_prophet_m2_k16_meta.txt    -- accuracy summary

Run:
  /c/Python314/python tools/train_m2_big.py
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
        if c == '/' and i + 1 < n and src[i+1] == '/':
            while i < n and src[i] != '\n':
                i += 1
            continue
        two = src[i:i+2]
        if two in TWO_CHAR:
            out.append(VOCAB.get(two, VOCAB['ID']))
            i += 2
            continue
        if c in (':', ',', ';', '{', '}', '(', ')', '+', '-', '*', '/', '=', '<', '>'):
            if c == '-' and i + 1 < n and src[i+1] == '>':
                out.append(VOCAB['->']); i += 2
                continue
            out.append(VOCAB[c]); i += 1
            continue
        if c.isdigit():
            j = i
            while j < n and src[j].isdigit():
                j += 1
            out.append(VOCAB['NUM']); i = j
            continue
        if c.isalpha() or c == '_':
            j = i
            while j < n and (src[j].isalnum() or src[j] == '_'):
                j += 1
            word = src[i:j]
            if word in KEYWORDS:
                out.append(VOCAB[word])
            else:
                out.append(VOCAB['ID'])
            i = j
            continue
        i += 1
    return out


def collect_corpus():
    parts = []
    SKIP_BIG = {'bc_src_c.kenga', 'more.kenga', 'lower_kv.kenga',
                'lower_c.kenga', 'rt_prophet.kenga', 'native_ml.kenga'}
    for root in ('kenga', 'examples'):
        for r, ds, fs in os.walk(root):
            for f in fs:
                if not f.endswith('.kenga'):
                    continue
                if f in SKIP_BIG:
                    continue
                if f.startswith('mid_prophet') or f.startswith('pico_birth'):
                    continue
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
        except Exception:
            continue
        tok = tokenize(src)
        big.extend(tok)
    print(f'total train tokens: {len(big)}')

    os.makedirs('minds', exist_ok=True)
    with open('minds/mid_prophet_m2_k16_vocab.txt', 'w') as f:
        f.write(f'# vocab tokens = {NUM_TOK}\n')
        for tok, idx in VOCAB.items():
            f.write(f'{idx}\t{tok}\n')

    n = len(big)
    split = int(n * 0.9)
    train_stream = big[:split]
    val_stream = big[split:]
    with open('minds/mid_prophet_m2_k16_train.txt', 'w') as f:
        f.write(' '.join(str(x) for x in train_stream))
    with open('minds/mid_prophet_m2_k16_test.txt', 'w') as f:
        f.write(' '.join(str(x) for x in val_stream))

    # numpy training: vectorized softmax-linear, K=window, V=28
    K = 16  # M2.1 explicit
    V = NUM_TOK
    rng = np.random.RandomState(7)
    W = rng.randn(V, K * V) * 0.02
    b = np.zeros(V)
    print(f'training: V={V} K={K} tot_params={V*(K*V)+V}')

    LR = 0.005
    epochs = 60
    # Adam
    b1, b2, eps = 0.9, 0.999, 1e-8
    mW = np.zeros_like(W)
    vW = np.zeros_like(W)
    mb = np.zeros_like(b)
    vb = np.zeros_like(b)
    n_total = len(train_stream)
    train_arr = np.array(train_stream, dtype=np.int32)
    val_arr = np.array(val_stream, dtype=np.int32)

    for ep in range(epochs):
        # Iterate all positions
        # windows shape = (n-K, K) -- each row is a window
        # Build all windows at once (memory ~ 200k x 4 x 4 = 3.2MB)
        if n_total - K <= 0:
            break
        idx = np.arange(K, n_total)
        wins = np.stack([train_arr[idx - K + j] for j in range(K)], axis=1)  # (n-K, K)
        # one-hot: position (n-K, K*V)
        one = np.zeros((wins.shape[0], K * V), dtype=np.float32)
        for j in range(K):
            one[np.arange(wins.shape[0]), wins[:, j]] = 1.0
        # forward: logits = one @ W.T + b   shape (n-K, V); targets (n-K,)
        logits = one @ W.T + b
        # softmax
        m = logits.max(axis=1, keepdims=True)
        exps = np.exp(logits - m)
        probs = exps / exps.sum(axis=1, keepdims=True)
        targets = train_arr[idx]
        # gradient
        g = probs.copy()
        g[np.arange(g.shape[0]), targets] -= 1.0
        # backward (Adam)
        dW = (g.T @ one) / g.shape[0]
        db = g.mean(axis=0)
        mW = b1 * mW + (1 - b1) * dW
        vW = b2 * vW + (1 - b2) * (dW * dW)
        mW_hat = mW / (1 - b1 ** (ep + 1))
        vW_hat = vW / (1 - b2 ** (ep + 1))
        W -= LR * mW_hat / (np.sqrt(vW_hat) + eps)
        mb = b1 * mb + (1 - b1) * db
        vb = b2 * vb + (1 - b2) * (db * db)
        mb_hat = mb / (1 - b1 ** (ep + 1))
        vb_hat = vb / (1 - b2 ** (ep + 1))
        b -= LR * mb_hat / (np.sqrt(vb_hat) + eps)

        # accuracy
        preds = probs.argmax(axis=1)
        acc = (preds == targets).mean() * 100
        print(f'  epoch {ep:>2d}: in-dist acc = {acc:.2f}%')

    # Save weights
    SCALE = 1000
    with open('minds/mid_prophet_m2_k16_w.txt', 'w') as f:
        f.write(f'vocab={V} k={K} scale={SCALE}\n')
        for v in range(V):
            row = [round(float(W[v, d]) * SCALE) for d in range(K * V)]
            f.write(f'[v={v}] ' + ','.join(str(int(w)) for w in row) + f',{round(float(b[v])*SCALE)}\n')

    # Held-out eval
    print(f'\n=== held-out evaluation ===')
    val_total = 0
    correct = 0
    for p in held_files:
        try:
            src = open(p, encoding='utf-8', errors='replace').read()
        except Exception:
            continue
        tok = tokenize(src)
        if len(tok) < K + 1:
            continue
        arr = np.array(tok, dtype=np.int32)
        idx = np.arange(K, len(tok))
        wins = np.stack([arr[idx - K + j] for j in range(K)], axis=1)
        one = np.zeros((wins.shape[0], K * V), dtype=np.float32)
        for j in range(K):
            one[np.arange(wins.shape[0]), wins[:, j]] = 1.0
        logits = one @ W.T + b
        preds = logits.argmax(axis=1)
        c = int((preds == arr[idx]).sum())
        t = int(idx.shape[0])
        val_total += t
        correct += c
        name = p.replace('\\\\', '/').split('/')[-1]
        print(f'  held {name}: {c}/{t} = {c*100/t:.2f}%')

    print(f'\nkenga-prophet m2.1 (K=16) held-out: {correct}/{val_total} = {correct*100/val_total:.2f}%')

    # Save summary
    with open('minds/mid_prophet_m2_k16_meta.txt', 'w') as f:
        f.write(f'V={V}\n')
        f.write(f'K={K}\n')
        f.write(f'train_tokens={n_total}\n')
        f.write(f'params={V*(K*V)+V}\n')
        f.write(f'epochs={epochs}\n')
        f.write(f'lr={LR}\n')
        f.write(f'held_total={val_total}\n')
        f.write(f'held_correct={correct}\n')


if __name__ == '__main__':
    main()
