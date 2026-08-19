"""tools/kenchat.py — generate-and-run pipeline for Kenga Prophet.

Second metric: program validity rate (compile + run + correct value).
"""
import argparse, os, sys, subprocess, tempfile
import numpy as np

VOCAB_TOKENS = [
    'fn', 'return', 'let', 'if', 'else', 'while', 'for', 'i64',
    ':', ',', ';', '{', '}', '(', ')', '->',
    '+', '-', '*', '/', '=', '==', '<', '<=', '>', 'println',
    'ID', 'NUM',
]
V = len(VOCAB_TOKENS)
K_LITE = 'bootstrap/bin/kenga-lite.exe'


def load_weights(path):
    """Load weights file. Format: header line 'vocab=N k=K scale=S'
    then one line per class: '[v=X] n1,n2,...' (each row is K*V+1 numbers)."""
    weights = {}
    with open(path) as f:
        for line in f:
            line = line.strip()
            if not line.startswith('[v='):
                continue
            rb = line.find(']')
            if rb < 0:
                continue
            name = line[1:rb].strip()  # 'v=0'
            body = line[rb+1:].strip()
            nums = []
            for x in body.split(','):
                x = x.strip()
                if not x: continue
                try: nums.append(int(round(float(x) / 1000.0)))
                except ValueError: pass
            weights[name] = np.array(nums, dtype=np.int32)
    return weights


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


def tokenize(src):
    KEYWORDS = {'fn','return','let','if','else','while','for','i64','println'}
    TWO_CHAR = {'->','==','<=','>=','!=','&&','||','<<','>>','&','|','^','~'}
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
            out.append(VOCAB_MAP['NUM']); i = j; continue
        if c.isalpha() or c == '_':
            j = i
            while j < n and (src[j].isalnum() or src[j] == '_'):
                j += 1
            word = src[i:j]
            out.append(VOCAB_MAP[word] if word in KEYWORDS else VOCAB_MAP['ID'])
            i = j; continue
        i += 1
    return out


def detokenize(toks):
    out = []
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
    return ''.join(out)


def predict_one(W_arr, V_, K, window):
    KV = V_ * K
    feat = np.zeros(KV, dtype=np.float32)
    for j, t in enumerate(window):
        if 0 <= t < V_:
            feat[j * V_ + t] = 1.0
    logits = feat @ W_arr[:, :KV].T + W_arr[:, KV]
    return int(np.argmax(logits))


def predict_sample(W_arr, V_, K, window, temperature=2.5):
    KV = V_ * K
    feat = np.zeros(KV, dtype=np.float32)
    for j, t in enumerate(window):
        if 0 <= t < V_:
            feat[j * V_ + t] = 1.0
    logits = (feat @ W_arr[:, :KV].T + W_arr[:, KV]).astype(np.float32)
    logits = logits / 1000.0
    logits = logits / temperature
    logits = logits - logits.max()
    exp = np.exp(logits)
    probs = exp / exp.sum()
    rng = np.random.default_rng()
    return int(rng.choice(V_, p=probs))


def gen_tokens(prompt, weights_path, max_tokens=80, temperature=None):
    info = read_header(weights_path)
    K = info.get('k', 8)
    W_dict = load_weights(weights_path)
    # Build W_arr: rows are v=0..V-1 of shape (K*V+1,)
    arr_dict = {int(k_.split('=', 1)[1]): v_ for k_, v_ in W_dict.items() if k_.startswith('v=')}
    if not arr_dict:
        return [], ''
    V_ = max(arr_dict.keys()) + 1
    # all rows must have same shape
    row0 = arr_dict[0]
    W_arr = np.zeros((V_, row0.shape[0]), dtype=np.int32)
    for v, arr in arr_dict.items():
        W_arr[v, :arr.shape[0]] = arr[:row0.shape[0]]

    toks = tokenize(prompt)
    if len(toks) > K:
        toks = toks[-K:]
    pad = [VOCAB_TOKENS.index('ID')] * (K - len(toks))
    toks = pad + toks

    generated = []
    n_braces = 0
    for step in range(max_tokens):
        if temperature:
            nxt = predict_sample(W_arr, V_, K, toks[-K:], temperature)
        else:
            nxt = predict_one(W_arr, V_, K, toks[-K:])
        generated.append(nxt)
        toks.append(nxt)
        if VOCAB_TOKENS[nxt] == '{':
            n_braces += 1
        elif VOCAB_TOKENS[nxt] == '}':
            n_braces -= 1
        if False:
            break
    return generated, detokenize(generated)


def run_via_kenga_lite(source, workdir='minds/_kenchat'):
    os.makedirs(workdir, exist_ok=True)
    f = tempfile.NamedTemporaryFile(mode='w', suffix='.kenga', delete=False,
                                    dir=workdir, encoding='utf-8')
    f.write(source)
    f.close()
    try:
        result = subprocess.run(
            [K_LITE, 'run', f.name],
            capture_output=True, text=True, timeout=60,
        )
        return result.returncode, result.stdout, result.stderr
    except subprocess.TimeoutExpired:
        return -1, '', 'timeout'
    except Exception as e:
        return -2, '', str(e)
    finally:
        os.unlink(f.name)


PROBES = [
    ("fn add",   "5"),
    ("fn sub",   "7"),
    ("fn mul",   "42"),
    ("fn fact",  "120"),
    ("fn fib",   "21"),
    ("fn max",   "7"),
    ("fn sqr",   "81"),
    ("fn pow",   "1024"),
    ("fn sumto", "55"),
]


def make_valid_program(prompt, generated_source):
    src = prompt + generated_source
    # Make sure src ends with newline + a main if not present
    if 'fn main' not in src:
        src = src + '\nfn main() -> i64 { println(0); return 0; }\n'
    else:
        src = src + '\n'
    return src


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('prompt', nargs='?', help='generate from prompt')
    ap.add_argument('--probe', action='store_true', help='run all 9 probes')
    ap.add_argument('--model', default='v01',
                    help='v01 (K=8) or k16 (K=16)')
    ap.add_argument('--max-tokens', type=int, default=80)
    ap.add_argument('--temperature', type=float, default=None,
                    help='sample with temperature (default: greedy argmax)')
    args = ap.parse_args()

    if args.model == 'v01':
        weights_path = 'minds/mid_prophet_m2_big_w.txt'
        label = 'kenga-prophet m2 v0.1 K=8'
    elif args.model == 'k16':
        weights_path = 'minds/mid_prophet_m2_k16_w.txt'
        label = 'kenga-prophet m2.1 K=16'
    else:
        print('unknown model', file=sys.stderr); sys.exit(1)

    print(f'using model: {label}')

    if args.prompt:
        toks, src = gen_tokens(args.prompt, weights_path, args.max_tokens,
                               args.temperature)
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
    n_compile = 0; n_run = 0; n_correct = 0
    for prompt, want in PROBES:
        _, src = gen_tokens(prompt, weights_path, args.max_tokens,
                            args.temperature)
        full = make_valid_program(prompt, src)
        rc, out, err = run_via_kenga_lite(full)
        first = out.strip().split('\n')[0] if out else ''
        ok_compile = rc in (0, 2)
        ok_run = rc == 0
        ok_value = first == want
        print(f'prompt={prompt!r:12s}  want={want:>5}  rc={rc:<3}  out={first[:20]:<20}  compile={ok_compile}  run={ok_run}  match={ok_value}')
        if ok_compile: n_compile += 1
        if ok_run: n_run += 1
        if ok_value: n_correct += 1
    total = len(PROBES)
    print()
    total = len(PROBES)
    print()
    sys.stdout.write('compile-ok:    ' + str(n_compile) + '/' + str(total) + ' = %.1f%%\n' % (n_compile*100/total))
    sys.stdout.write('run-ok:        ' + str(n_run) + '/' + str(total) + ' = %.1f%%\n' % (n_run*100/total))
    sys.stdout.write('match value:   ' + str(n_correct) + '/' + str(total) + ' = %.1f%%\n' % (n_correct*100/total))
    sys.stdout.flush()
    return 0


if __name__ == '__main__':
    sys.exit(main())

def _debug_gen(prompt='fn add(a: i64, b: i64) -> i64 {'):
    toks, src = gen_tokens(prompt, 'minds/mid_prophet_m2_big_w.txt', 80)
    print('==', prompt, '=> tokens =', toks[:40])
    print('source:')
    print(src)
