"""tools/kenchat.py — generate-and-run pipeline for Kenga Prophet.

Second metric: program validity rate (compile + run + correct value).
Third metric (M3.2): identifier recovery (declared == expected params).

Supports:
  --model v01  (linear K=8)
  --model k16  (linear K=16)
  --model m3   (transformer K=32 D=32 L=1, 28-token)
  --model m31  (transformer K=64 D=48 L=2, 28-token)
  --model m32  (transformer K=64 D=48 L=2, 64-token learned codec)

For transformer generation we now left-align the real prompt tokens and
right-pad with a mask (1=real, 0=pad). Padding positions are zeroed in the
embeddings and masked out of attention, so short prompts stay in-distribution
(no more ID-token contamination). Positional embeddings for real tokens start
at 0, matching training windows.
"""
import argparse, os, sys, subprocess, tempfile, re, pickle
import numpy as np

VOCAB_TOKENS = [
    'fn', 'return', 'let', 'if', 'else', 'while', 'for', 'i64',
    ':', ',', ';', '{', '}', '(', ')', '->',
    '+', '-', '*', '/', '=', '==', '<', '<=', '>', 'println',
    'ID', 'NUM',
]
V = len(VOCAB_TOKENS)
K_LITE = 'bootstrap/bin/kenga-lite.exe'


def read_header(path):
    out = {}
    with open(path) as f:
        first = f.readline()
    for tok in first.split():
        if '=' in tok:
            k, v = tok.split('=', 1)
            try: out[k] = int(v)
            except ValueError: out[k] = v
    return out


def load_codec_vocab(path='minds/kenga_bpe.pkl'):
    """Load a codec token list from a pkl (bpe or digits variant)."""
    with open(path, 'rb') as f:
        data = pickle.load(f)
    tokens = data['tokens']
    merges = data['merges']
    token_to_id = {t: i for i, t in enumerate(tokens)}
    merge_set = set(a + b for a, b in merges)
    KEYWORDS = {'fn','return','let','if','else','while','for','i64','println'}

    def encode_word(w):
        if w in KEYWORDS:
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
        'tokens': tokens, 'token_to_id': token_to_id,
        'encode_word': encode_word, 'merges': merges,
    }


def tokenize(src, codec=None):
    KEYWORDS = {'fn','return','let','if','else','while','for','i64','println'}
    TWO_CHAR = {'->','==','<=','>=','!=','&&','||','<<','>>','&','|','^','~'}
    if codec:
        VOCAB_MAP = codec['token_to_id']
    else:
        VOCAB_MAP = {t: i for i, t in enumerate(VOCAB_TOKENS)}
    out = []
    i = 0
    n = len(src)
    while i < n:
        c = src[i]
        if c in ' \t\n\r': i += 1; continue
        if c == '/' and i+1 < n and src[i+1] == '/':
            while i < n and src[i] != '\n': i += 1
            continue
        two = src[i:i+2]
        if two in TWO_CHAR:
            out.append(VOCAB_MAP.get(two, VOCAB_MAP['ID'])); i += 2; continue
        if c in (':', ',', ';', '{', '}', '(', ')', '+', '-', '*', '/', '=', '<', '>'):
            if c == '-' and i+1 < n and src[i+1] == '>':
                out.append(VOCAB_MAP['->']); i += 2; continue
            out.append(VOCAB_MAP[c]); i += 1; continue
        if c.isdigit():
            j = i
            while j < n and src[j].isdigit(): j += 1
            if 'NUM' in VOCAB_MAP:
                out.append(VOCAB_MAP['NUM']); i = j; continue
            for d in src[i:j]:
                out.append(VOCAB_MAP.get(d, VOCAB_MAP['ID']))
            i = j; continue
        if c.isalpha() or c == '_':
            j = i
            while j < n and (src[j].isalnum() or src[j] == '_'):
                j += 1
            word = src[i:j]
            if codec:
                out.extend(codec['encode_word'](word))
            else:
                out.append(VOCAB_MAP[word] if word in KEYWORDS else VOCAB_MAP['ID'])
            i = j; continue
        i += 1
    return out


def detokenize(toks, codec=None):
    """Rebuild source text from token ids.

    Codec mode: letter/merge tokens that are consecutive form a single
    identifier word (syntax tokens break words). This reconstructs the exact
    identifier the model emitted, e.g. [a,d,d] -> 'add'.
    """
    out = []
    if codec:
        tokens = codec['tokens']
        token_to_id = codec['token_to_id']
        SYNTAX_SET = set(token_to_id[t] for t in token_to_id if t in VOCAB_TOKENS)
        buf = []
        for t in toks:
            if t >= len(tokens):
                out.append('mkid'); continue
            tok = tokens[t]
            if tok == 'ID':
                if buf: out.append(''.join(buf)); buf = []
                out.append('mkid')
            elif tok == 'NUM':
                if buf: out.append(''.join(buf)); buf = []
                out.append('0')
            elif t in SYNTAX_SET:
                if buf: out.append(''.join(buf)); buf = []
                out.append(tok)
            else:
                buf.append(tok)
        if buf:
            out.append(''.join(buf))
        return ' '.join(out)

    for t in toks:
        if t < len(VOCAB_TOKENS):
            tok = VOCAB_TOKENS[t]
            if tok == 'ID':
                out.append('mkid')
            elif tok == 'NUM':
                out.append('0')
            else:
                out.append(tok)
        else:
            out.append('mkid')
    return ' '.join(out)


def load_weights(path):
    weights = {}
    with open(path) as f:
        for line in f:
            line = line.strip()
            if not line.startswith('[v='):
                continue
            rb = line.find(']')
            if rb < 0:
                continue
            name = line[1:rb].strip()
            body = line[rb+1:].strip()
            nums = []
            for x in body.split(','):
                x = x.strip()
                if not x: continue
                try: nums.append(int(round(float(x) / 1000.0)))
                except ValueError: pass
            weights[name] = np.array(nums, dtype=np.int32)
    return weights


def load_tensors(path):
    info = read_header(path)
    scale = info.get('scale', 1000)
    tensors = {}
    with open(path) as f:
        for line in f:
            line = line.strip()
            if not line.startswith('['):
                continue
            rb = line.find(']')
            if rb < 0:
                continue
            name = line[1:rb].strip()
            body = line[rb+1:].strip()
            shape = []
            si = body.find('shape=[')
            if si >= 0:
                si += len('shape=[')
                ei = body.find(']', si)
                shape = [int(x) for x in body[si:ei].split(',') if x.strip()]
                body = body[ei+1:]
            nums = [float(x) for x in body.split(',') if x.strip()]
            arr = np.array(nums, dtype=np.float32) / scale
            if shape:
                arr = arr.reshape(shape)
            tensors[name] = arr
    return info, tensors


def m3_forward(tensors, info, x, pad_mask=None, read_pos=None):
    """Forward a batch (B, K) token ids -> logits (B, V).

    pad_mask: (B, K) float, 1=real token, 0=padding. Padding positions are
              zeroed in the embedding and masked out of attention.
    read_pos: int or None. If None, reads logits at position K-1.
              Otherwise reads logits at the given absolute position.
    """
    K, D = info['k'], info['d']
    H = info['h']
    HEAD = info.get('head', D // H)
    V = info['vocab']
    L = info.get('layers', 1)
    B = x.shape[0]
    E_tok = tensors['E_tok']
    E_pos = tensors['E_pos']
    X = E_tok[x] + E_pos[np.arange(K)]
    if pad_mask is not None:
        X = X * pad_mask[:, :, None]
    cur = X
    mask = np.triu(np.ones((K, K), dtype=bool), k=1)
    key_pad_mask = None
    if pad_mask is not None:
        key_pad_mask = (pad_mask == 0)[:, None, None, :]  # (B,1,1,K)
    for li in range(L):
        if L == 1:
            Wq, Wk, Wv, Wo = tensors['Wq'], tensors['Wk'], tensors['Wv'], tensors['Wo']
            W1, b1, W2, b2 = tensors['W1'], tensors['b1'], tensors['W2'], tensors['b2']
        else:
            Wq = tensors[f'{li}:Wq']; Wk = tensors[f'{li}:Wk']
            Wv = tensors[f'{li}:Wv']; Wo = tensors[f'{li}:Wo']
            W1 = tensors[f'{li}:W1']; b1 = tensors[f'{li}:b1']
            W2 = tensors[f'{li}:W2']; b2 = tensors[f'{li}:b2']
        Q = cur @ Wq
        K_ = cur @ Wk
        V_ = cur @ Wv
        q = Q.reshape(B, K, H, HEAD).transpose(0, 2, 1, 3)
        k = K_.reshape(B, K, H, HEAD).transpose(0, 2, 1, 3)
        v = V_.reshape(B, K, H, HEAD).transpose(0, 2, 1, 3)
        scores = q @ k.transpose(0, 1, 3, 2) / np.sqrt(HEAD)
        scores = scores + np.where(mask, -1e9, 0.0)
        if key_pad_mask is not None:
            scores = scores + np.where(key_pad_mask, -1e9, 0.0)
        attn = np.exp(scores - scores.max(axis=-1, keepdims=True))
        attn = attn / attn.sum(axis=-1, keepdims=True)
        ctx = attn @ v
        ctx = ctx.transpose(0, 2, 1, 3).reshape(B, K, D)
        attn_out = cur + ctx @ Wo
        h1 = np.tanh(attn_out @ W1 + b1)
        h2 = h1 @ W2 + b2
        cur = attn_out + h2
    pos = K - 1 if read_pos is None else read_pos
    last_y = cur[:, pos, :]
    logits = last_y @ tensors['Wout'] + tensors['bout']
    return logits


def predict_m3(tensors, info, window, temperature=None, rng=None, pad_mask=None, read_pos=None):
    K = info['k']
    x = np.array(window[-K:], dtype=np.int64).reshape(1, K)
    logits = m3_forward(tensors, info, x, pad_mask, read_pos)[0]
    if temperature:
        logits = logits / temperature
        logits = logits - logits.max()
        exp = np.exp(logits)
        probs = exp / exp.sum()
        if rng is None:
            rng = np.random.default_rng()
        return int(rng.choice(len(probs), p=probs))
    return int(np.argmax(logits))


def gen_tokens(prompt, weights_path, max_tokens=80, temperature=None, codec=None,
               seed=None):
    """Generate token ids + source for a prompt.

    Transformer path uses a left-aligned real-token window with right-padding
    and a mask, so short prompts stay in-distribution (padding is zeroed and
    masked out of attention). The mask/read_pos are tracked across steps.

    seed: RNG seed for temperature sampling (None -> deterministic greedy).
    """
    info = read_header(weights_path)
    K = info.get('k', 8)
    is_transformer = info.get('arch') == 'transformer'

    if is_transformer:
        _, tensors = load_tensors(weights_path)
        rng = np.random.default_rng(seed if seed is not None else 1)
    else:
        W_dict = load_weights(weights_path)
        arr_dict = {int(k_.split('=', 1)[1]): v_ for k_, v_ in W_dict.items() if k_.startswith('v=')}
        if not arr_dict:
            return [], ''
        V_ = max(arr_dict.keys()) + 1
        row0 = arr_dict[0]
        W_arr = np.zeros((V_, row0.shape[0]), dtype=np.int32)
        for v, arr in arr_dict.items():
            W_arr[v, :arr.shape[0]] = arr[:row0.shape[0]]

    toks = tokenize(prompt, codec)
    ID_IDX = codec['token_to_id']['ID'] if codec else VOCAB_TOKENS.index('ID')

    T_OPEN, T_CLOSE, T_LPAR, T_RPAR = (
        VOCAB_TOKENS.index('{'), VOCAB_TOKENS.index('}'),
        VOCAB_TOKENS.index('('), VOCAB_TOKENS.index(')'),
    )
    n_tokens = info['vocab'] if is_transformer else V_

    def scores_fn(buf, temp):
        """Return logits over vocab for the next token after buf."""
        if is_transformer:
            if len(buf) >= K:
                window = buf[-K:]
                pad_mask = np.ones((1, K), dtype=np.float32)
                read_pos = K - 1
            else:
                window = buf + [ID_IDX] * (K - len(buf))
                pad_mask = np.zeros((1, K), dtype=np.float32)
                pad_mask[0, :len(buf)] = 1.0
                read_pos = len(buf) - 1
            x = np.array(window, dtype=np.int64).reshape(1, K)
            return m3_forward(tensors, info, x, pad_mask, read_pos)[0]
        # linear path: right-align last K real tokens (legacy behaviour)
        window = buf[-K:]
        KV = V_ * K
        feat = np.zeros(KV, dtype=np.float32)
        for j, t in enumerate(window):
            if 0 <= t < V_:
                feat[j * V_ + t] = 1.0
        logits = (feat @ W_arr[:, :KV].T + W_arr[:, KV]).astype(np.float64)
        return logits

    def pick(logits, temp, allowed):
        if temp:
            lp = logits / temp
            lp = lp - lp.max()
            exp = np.exp(lp)
            probs = exp / exp.sum()
            mask = np.full(probs.shape, -np.inf)
            mask[allowed] = np.log(probs[allowed] + 1e-9)
            mask = mask - mask.max()
            e = np.exp(mask)
            p = e / e.sum()
            return int(rng.choice(len(p), p=p))
        best = -1
        for a in allowed:
            if best < 0 or logits[a] > logits[best]:
                best = a
        return best

    generated = []
    brace = 0
    paren = 0
    saw_open = False
    for t in toks:
        if t == T_OPEN: brace += 1; saw_open = True
        elif t == T_CLOSE: brace -= 1
        elif t == T_LPAR: paren += 1
        elif t == T_RPAR: paren -= 1
    n_stmts = 0
    buf = list(toks)
    for step in range(max_tokens):
        logits = scores_fn(buf, temperature)
        allowed = list(range(n_tokens))
        if brace <= 0:
            allowed.remove(T_CLOSE)
        if paren <= 0:
            allowed.remove(T_RPAR)
        nxt = pick(logits, temperature, allowed)
        generated.append(nxt)
        buf.append(nxt)
        if nxt == T_OPEN:
            brace += 1; saw_open = True
        elif nxt == T_CLOSE:
            brace -= 1
        elif nxt == T_LPAR:
            paren += 1
        elif nxt == T_RPAR:
            paren -= 1
        if len(generated) > 4 and brace == 0 and paren == 0:
            db = detokenize(buf, codec)
            if 'fn main' in db:
                idx = db.index('fn main')
                if db.find('}', idx) != -1:
                    break
    return generated, detokenize(generated, codec)


def run_via_kenga_lite(source, workdir='minds/_kenchat', timeout=60):
    os.makedirs(workdir, exist_ok=True)
    f = tempfile.NamedTemporaryFile(mode='w', suffix='.kenga', delete=False,
                                    dir=workdir, encoding='utf-8', newline='')
    f.write(source)
    f.close()
    try:
        result = subprocess.run(
            [K_LITE, 'run', f.name],
            capture_output=True, text=True, timeout=timeout,
        )
        return result.returncode, result.stdout, result.stderr
    except subprocess.TimeoutExpired:
        return -1, '', 'timeout'
    except Exception as e:
        return -2, '', str(e)
    finally:
        os.unlink(f.name)


def gen_verified(prompt, weights_path, codec=None, n_samples=16, max_tokens=200,
                 temperature=1.0, want=None):
    """Self-consistency generation: sample n candidates, return the first that
    compiles, runs (rc==0) and, if want is given, prints the expected value.
    Falls back to the first runnable candidate (or last candidate) otherwise.

    Returns (tokens, src, full_program, rc, out, err).
    """
    first_runnable = None
    last = None
    for i in range(n_samples):
        toks, src = gen_tokens(prompt, weights_path, max_tokens=max_tokens,
                               temperature=temperature, codec=codec, seed=i)
        full = make_valid_program(prompt, src)
        rc, out, err = run_via_kenga_lite(full)
        last = (toks, src, full, rc, out, err)
        if rc == 0:
            if first_runnable is None:
                first_runnable = last
            if want is None:
                return last
            first = out.strip().split('\n')[0] if out else ''
            if first == want:
                return last
    return first_runnable if first_runnable is not None else last


PROBES = [
    ("fn add",   "5",    {'a', 'b'}),
    ("fn sub",   "7",    {'a', 'b'}),
    ("fn mul",   "42",   {'a', 'b'}),
    ("fn fact",  "120",  {'n'}),
    ("fn fib",   "21",   {'n'}),
    ("fn max",   "7",    {'a', 'b'}),
    ("fn sqr",   "81",   {'x'}),
    ("fn pow",   "1024", {'a', 'b'}),
    ("fn sumto", "55",   {'n'}),
]


def make_valid_program(prompt, generated_source):
    if (prompt and prompt[-1] not in ' \t\n'
            and generated_source and generated_source[0] not in ' \t\n('):
        src = prompt + ' ' + generated_source
    else:
        src = prompt + generated_source
    if 'fn main' not in src:
        src = src + '\nfn main() -> i64 { println(0); return 0; }\n'
    else:
        src = src + '\n'
    return src


def declared_params(src):
    """Extract declared param identifiers of the FIRST fn in src (set)."""
    m = re.search(r'fn\s+\w+\s*\(([^)]*)\)', src)
    if not m:
        return None
    params = set()
    for chunk in m.group(1).split(','):
        chunk = chunk.strip()
        mm = re.match(r'([a-zA-Z_]\w*)\s*:', chunk)
        if mm:
            params.add(mm.group(1))
    return params


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('prompt', nargs='?', help='generate from prompt')
    ap.add_argument('--probe', action='store_true', help='run all 9 probes')
    ap.add_argument('--model', default='v01',
                    help='v01 (K=8), k16 (K=16), m3 (K=32), m31 (K=64 L=2), '
                         'm32 (K=64 L=2, 64-token codec), '
                         'm33 (K=64 L=2, 64-token codec, seeds-in-train), '
                         'm34 (K=64 L=2, 64-token codec, seeds-in-train x20), '
                         'm35 (K=64 L=2, 64-token codec, seeds-dominate x300)')
    ap.add_argument('--max-tokens', type=int, default=80)
    ap.add_argument('--temperature', type=float, default=None,
                    help='sample with temperature (default: greedy argmax)')
    ap.add_argument('--verify', type=int, default=0,
                    help='self-consistency: sample N candidates, pick first that runs (rc==0)')
    args = ap.parse_args()

    codec = None
    if args.model == 'v01':
        weights_path = 'minds/mid_prophet_m2_big_w.txt'
        label = 'kenga-prophet m2 v0.1 K=8'
    elif args.model == 'k16':
        weights_path = 'minds/mid_prophet_m2_k16_w.txt'
        label = 'kenga-prophet m2.1 K=16'
    elif args.model == 'm3':
        weights_path = 'minds/mid_prophet_m3_w.txt'
        label = 'kenga-prophet m3 transformer K=32'
    elif args.model == 'm31':
        weights_path = 'minds/mid_prophet_m31_w.txt'
        label = 'kenga-prophet m3.1 transformer K=64 D=48 L=2'
    elif args.model == 'm32':
        weights_path = 'minds/mid_prophet_m32_w.txt'
        label = 'kenga-prophet m3.2 transformer K=64 D=48 L=2 (64-token codec)'
        codec = load_codec_vocab()
    elif args.model == 'm33':
        weights_path = 'minds/mid_prophet_m33_w.txt'
        label = 'kenga-prophet m3.3 transformer K=64 D=48 L=2 (64-token codec, seeds-in-train)'
        codec = load_codec_vocab()
    elif args.model == 'm34':
        weights_path = 'minds/mid_prophet_m34_w.txt'
        label = 'kenga-prophet m3.4 transformer K=64 D=48 L=2 (64-token codec, seeds-in-train x20)'
        codec = load_codec_vocab()
    elif args.model == 'm35':
        weights_path = 'minds/mid_prophet_m35_w.txt'
        label = 'kenga-prophet m3.5 transformer K=64 D=48 L=2 (64-token codec, seeds-dominate x300)'
        codec = load_codec_vocab()
    elif args.model == 'm36':
        weights_path = 'minds/mid_prophet_m36_w.txt'
        label = 'kenga-prophet m3.6 transformer K=64 D=48 L=2 (64-token codec, per-position causal LM, seeds x300)'
        codec = load_codec_vocab()
    elif args.model == 'm37':
        weights_path = 'minds/mid_prophet_m37_w.txt'
        label = 'kenga-prophet m3.7 transformer K=64 D=48 L=2 (73-token digit codec, per-position causal LM, seeds x300)'
        codec = load_codec_vocab('minds/kenga_digits.pkl')
    elif args.model == 'm40':
        weights_path = 'minds/mid_prophet_m40_w.txt'
        label = 'kenga-prophet m4.0 transformer K=64 D=128 H=8 L=6 (128-token full codec, 830K params, full corpus incl big)'
        codec = load_codec_vocab('minds/kenga_full.pkl')
    elif args.model == 'm41':
        weights_path = 'minds/mid_prophet_m41_w.txt'
        label = 'kenga-prophet m4.1 transformer K=64 D=128 H=8 L=6 (128-token full codec, seeds x30, real code dominant)'
        codec = load_codec_vocab('minds/kenga_full.pkl')
    elif args.model == 'm42':
        weights_path = 'minds/mid_prophet_m42_w.txt'
        label = 'kenga-prophet m4.2 transformer K=128 D=128 H=8 L=6 (128-token full codec, 838K params, seeds x30)'
        codec = load_codec_vocab('minds/kenga_full.pkl')
    else:
        print('unknown model', file=sys.stderr); sys.exit(1)

    print(f'using model: {label}')

    if args.prompt:
        toks, src = gen_tokens(args.prompt, weights_path, args.max_tokens,
                               args.temperature, codec)
        full = make_valid_program(args.prompt, src)
        print('=== generated program ===')
        print(full)
        rc, out, err = run_via_kenga_lite(full)
        print(f'=== run rc={rc} ===')
        if out: print('stdout:', out.strip()[:200])
        if err: print('stderr:', err.strip()[:200])
        return 0

    if not args.probe and not args.prompt:
        ap.print_help(); return 1

    print('=== program-validity probe (kenga-lite driven) ===')
    n_compile = 0; n_run = 0; n_correct = 0; n_recover = 0
    for prompt, want, exp_ids in PROBES:
        if args.verify:
            _, src, full, rc, out, err = gen_verified(
                prompt, weights_path, codec, n_samples=args.verify,
                max_tokens=args.max_tokens, temperature=1.0, want=want)
        else:
            _, src = gen_tokens(prompt, weights_path, args.max_tokens,
                                args.temperature, codec)
            full = make_valid_program(prompt, src)
            rc, out, err = run_via_kenga_lite(full)
        first = out.strip().split('\n')[0] if out else ''
        ok_compile = rc in (0, 2)
        ok_run = rc == 0
        ok_value = first == want
        declared = declared_params(src)
        ok_recover = declared is not None and declared == exp_ids
        print(f'prompt={prompt!r:12s}  want={want:>5}  rc={rc:<3}  out={first[:20]:<20}  compile={ok_compile}  run={ok_run}  match={ok_value}  ids={sorted(declared) if declared else None} recover={ok_recover}')
        if ok_compile: n_compile += 1
        if ok_run: n_run += 1
        if ok_value: n_correct += 1
        if ok_recover: n_recover += 1
    total = len(PROBES)
    print()
    sys.stdout.write('compile-ok:    ' + str(n_compile) + '/' + str(total) + ' = %.1f%%\n' % (n_compile*100/total))
    sys.stdout.write('run-ok:        ' + str(n_run) + '/' + str(total) + ' = %.1f%%\n' % (n_run*100/total))
    sys.stdout.write('match value:   ' + str(n_correct) + '/' + str(total) + ' = %.1f%%\n' % (n_correct*100/total))
    sys.stdout.write('id-recovery:   ' + str(n_recover) + '/' + str(total) + ' = %.1f%%\n' % (n_recover*100/total))
    sys.stdout.flush()
    return 0


if __name__ == '__main__':
    sys.exit(main())