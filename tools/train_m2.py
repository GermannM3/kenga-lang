#!/usr/bin/env python3
"""
M2: opcode-token predictor trainer.

We define a 27-token codec of Kenga lexemes (token IDs 0..26).
For each of 9 kenga_seed_*.kenga files we encode -> token-id stream.
Train a tiny linear classifier: P(next | window of last 4 tokens).
Output weights as text file consumable by `examples/ml/mid_prophet_m2.kenga`.

Run:
  tools/train_m2.py          # writes minds/mid_prophet_m2_w.txt, _id.txt
"""

import os
import re

SEEDS = [
    "examples/ml/kenga_seed_add.kenga",
    "examples/ml/kenga_seed_sub.kenga",
    "examples/ml/kenga_seed_mul.kenga",
    "examples/ml/kenga_seed_fact.kenga",
    "examples/ml/kenga_seed_fib.kenga",
    "examples/ml/kenga_seed_max.kenga",
    "examples/ml/kenga_seed_sqr.kenga",
    "examples/ml/kenga_seed_pow.kenga",
    "examples/ml/kenga_seed_sum.kenga",
]

# Token vocabulary: token-id maps to lexeme for encoding/decoding
TOKENS = [
    "fn",     # 0
    "return", # 1
    "let",    # 2
    "if",     # 3
    "else",   # 4
    "while",  # 5
    "for",    # 6
    "i64",    # 7
    ":",      # 8
    ",",      # 9
    ";",      # 10
    "{",      # 11
    "}",      # 12
    "(",      # 13
    ")",      # 14
    "->",     # 15
    "+",      # 16
    "-",      # 17
    "*",      # 18
    "/",      # 19
    "=",      # 20
    "==",     # 21
    "<",      # 22
    "<=",     # 23
    ">",      # 24
    "println", # 25
    "ID",     # 26 identifier
    "NUM",    # 27 integer literal
]

# Special tokens at indices 0..N:
# We give them indices in dict.order.
VOCAB = {tok: i for i, tok in enumerate(TOKENS)}
NUM_TOK = len(TOKENS)


def tokenize(src: str) -> list:
    """Tokenize Kenga source into list of token ids."""
    out = []
    i = 0
    keywords = {"fn", "return", "let", "if", "else", "while", "for", "i64", "println"}
    two_char = {"->", "==", "<=", ">="}

    while i < len(src):
        c = src[i]
        # Skip whitespace, line comments
        if c in (" ", "\t", "\n", "\r"):
            i += 1
            continue
        if c == "/" and i + 1 < len(src) and src[i + 1] == "/":
            while i < len(src) and src[i] != "\n":
                i += 1
            continue
        # Two-char operators first
        two = src[i:i+2]
        if two in two_char:
            out.append(VOCAB[two])
            i += 2
            continue
        # Single-char symbols
        if c in (":", ",", ";", "{", "}", "(", ")", "+", "-", "*", "/", "=", "<", ">"):
            # Note: "-" and "/" need careful handling with "--" "->", "//"
            if c == "-" and i + 1 < len(src) and src[i+1] == ">":
                out.append(VOCAB["->"])
                i += 2
                continue
            if c == "/" and i + 1 < len(src) and src[i+1] == "/":
                i += 1  # already handled in comment branch
                continue
            out.append(VOCAB[c])
            i += 1
            continue
        # Number
        if c.isdigit():
            # Read the whole number literal
            j = i
            while j < len(src) and (src[j].isdigit()):
                j += 1
            out.append(VOCAB["NUM"])
            i = j
            continue
        # Identifier / keyword
        if c.isalpha() or c == "_":
            j = i
            while j < len(src) and (src[j].isalnum() or src[j] == "_"):
                j += 1
            word = src[i:j]
            # Strip common descriptor bit: see what follows to disambiguate
            # Handle case where src[i+2:] is a digit (e.g. "fn" is keyword vs identifier "fn_X")
            # Simple: if word in known keywords, output keyword; else ID.
            if word in keywords:
                out.append(VOCAB[word])
                out.append(VOCAB["ID"])
            i = j
            continue
        # Otherwise skip
        i += 1
    return out


def write_vocab(path: str):
    with open(path, "w") as f:
        f.write(f"# vocab tokens = {NUM_TOK}\n")
        for tok, idx in VOCAB.items():
            f.write(f"{idx}\t{tok}\n")


def write_stream(path: str, ids: list):
    """Write a token-id stream as space-separated ints (last token first, padded)."""
    with open(path, "w") as f:
        f.write(" ".join(str(x) for x in ids))


def main():
    os.makedirs("minds", exist_ok=True)
    write_vocab("minds/mid_prophet_m2_vocab.txt")
    print(f"vocab: {NUM_TOK} tokens")

    # Use first 5 seeds as training; allocate 4 seeds to held-out probe
    train_seeds = SEEDS[:5]
    held_seeds = SEEDS[5:]

    train_streams = {}
    for s in train_seeds + held_seeds:
        with open(s, "r") as f:
            src = f.read()
        ids = tokenize(src)
        train_streams[s] = ids
        print(f"  {s}:  tokens={len(ids)}")

    # Train: tiny linear softmax over (last K tokens one-hot concat) -> next token logit.
    # Hand-rolled, no numpy. K=4, V=28. Weights = V * (K*V) = 28 * (4*28) = 3136 params.
    K = 4
    V = NUM_TOK
    W = [[0.0] * (K * V) for _ in range(V)]
    b = [0.0] * V

    def features(window: list) -> list:
        # one-hot of last K tokens, concatenated
        f = [0] * (K * V)
        for j, tok in enumerate(window):
            f[j * V + tok] = 1
        return f

    def softmax(logits: list) -> list:
        m = max(logits)
        exps = [2.718281828459045 ** (l - m) for l in logits]
        s = sum(exps)
        return [e / s for e in exps]

    LR = 0.5
    epochs = 60
    for ep in range(epochs):
        # Shuffle seeds each epoch
        order = list(range(len(train_seeds)))
        # Simple shuffle: rotate
        order = order[1:] + order[:1]
        for idx in order:
            ids = train_streams[train_seeds[idx]]
            for pos in range(K, len(ids)):
                window = ids[pos - K:pos]
                target = ids[pos]
                feat = features(window)
                # logits = W*feat + b
                logits = [sum(W[v][d] * feat[d] for d in range(K * V)) + b[v] for v in range(V)]
                probs = softmax(logits)
                if probs[target] <= 0:
                    # loss skipped
                    import math
                # gradient: d_loss = probs[target] - 1 for target; probs[v] for others
                for v in range(V):
                    g = probs[v] - (1.0 if v == target else 0.0)
                    for d in range(K * V):
                        W[v][d] -= LR * g * feat[d]
                    b[v] -= LR * g
        if ep % 10 == 0:
            print(f"epoch {ep}: ok (loss approx)")

    # Save weights: rows of K*V+1 (with bias)
    # Format readable by Kenga: integer scaled by SCALE.
    # We use SCALE = 1000 so weights like -0.082830 become -83.
    # Kenga Lit more VM only has i64. The Python side keeps float precision.
    SCALE = 1000
    with open("minds/mid_prophet_m2_w.txt", "w") as f:
        f.write(f"vocab={V} k={K} scale={SCALE}\n")
        for v in range(V):
            row = [round(W[v][d] * SCALE) for d in range(K * V)]
            f.write(f"[v={v}] " + ",".join(str(int(w)) for w in row) + f",{round(b[v]*SCALE)}\n")

    # Save held-out streams
    write_stream("minds/mid_prophet_m2_train.txt", train_streams[train_seeds[0]])
    for s in held_seeds:
        name = s.split("/")[-1].replace(".kenga", "").replace("kenga_seed_", "")
        with open(f"minds/mid_prophet_m2_held_{name}.txt", "w") as wf:
            wf.write(" ".join(str(x) for x in train_streams[s]))

    # Compute training accuracy
    correct_train = 0
    total_train = 0
    for s in train_seeds:
        ids = train_streams[s]
        for pos in range(K, len(ids)):
            window = ids[pos - K:pos]
            target = ids[pos]
            feat = features(window)
            logits = [sum(W[v][d] * feat[d] for d in range(K * V)) + b[v] for v in range(V)]
            pred = max(range(V), key=lambda x: logits[x])
            if pred == target:
                correct_train += 1
            total_train += 1
    print(f"training next-token accuracy: {correct_train}/{total_train}")

    # Held-out next-token
    for s in held_seeds:
        ids = train_streams[s]
        correct = 0
        total = 0
        for pos in range(K, len(ids)):
            window = ids[pos - K:pos]
            target = ids[pos]
            feat = features(window)
            logits = [sum(W[v][d] * feat[d] for d in range(K * V)) + b[v] for v in range(V)]
            pred = max(range(V), key=lambda x: logits[x])
            if pred == target:
                correct += 1
            total += 1
        name = s.split("/")[-1].replace(".kenga", "").replace("kenga_seed_", "")
        print(f"  held {name}: {correct}/{total}")


if __name__ == "__main__":
    main()
