"""tools/realgen_eval.py — v2 two-tier generation eval on REAL held-out files.

Tier 1 (controlled semantic completion):
    prompt = the file's complete fn main block verbatim (arguments fixed
    by the human author); the model generates the missing definitions.
    kenga-lite accepts forward references (verified), so main-first order
    is valid. Semantic match = stdout equals the ORIGINAL file's stdout.
    Fair: no free choice of constants is left to the model.

Tier 2 (free continuation):
    prompt = first fn block (as v1). Measures valid-program rate only
    (compile/run); exact stdout conflates ability with arbitrary
    constant choices, reported but not gating.
"""
import argparse
import hashlib
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kenchat

BSLASH = chr(92)


def norm(p):
    return p.replace(BSLASH, '/')


def held_files(frac=0.1):
    SKIP_BIG = {
        'bc_src_c.kenga', 'more.kenga', 'lower_kv.kenga', 'lower_c.kenga',
        'rt_prophet.kenga', 'native_ml.kenga', 'rt_vm.kenga',
        'rt_tensor.kenga', 'rt_kval_tape.kenga', 'rt_kval_mem.kenga',
    }
    out = []
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
                h = int(hashlib.md5(norm(p).encode()).hexdigest(), 16) % 10000
                if h < frac * 10000:
                    out.append(p)
    return sorted(out)


def top_level_blocks(src):
    """Yield (start, end_exclusive) spans of each top-level braced block."""
    blocks = []
    depth = 0
    start = None
    for i, ch in enumerate(src):
        if ch == '{':
            if depth == 0:
                start = i
            depth += 1
        elif ch == '}':
            depth -= 1
            if depth == 0 and start is not None:
                blocks.append((start, i + 1))
                start = None
    return blocks


def header_start(src, brace_pos):
    """Index just after the newline preceding the line containing brace_pos."""
    nl = src.rfind('\n', 0, brace_pos)
    return 0 if nl < 0 else nl + 1


def block_with_header(src, span):
    s, e = span
    return src[header_start(src, s):e]


def extract_main(src):
    """Full text of the fn main block incl header, or None."""
    for s, e in top_level_blocks(src):
        blk = block_with_header(src, (s, e))
        if blk.lstrip().startswith('fn main'):
            return blk
    return None


def first_fn_block(src):
    for s, e in top_level_blocks(src):
        blk = block_with_header(src, (s, e))
        if not blk.lstrip().startswith('fn main'):
            return blk
    return src


def run_candidate(full_text, want, timeout=10):
    rc, out, _ = kenchat.run_via_kenga_lite(full_text, timeout=timeout)
    first = out.strip().split('\n')[0] if out.strip() else ''
    return rc == 0, (first == want), first


def gen_and_score(prompt, want, weights, codec, max_tokens, k):
    """Greedy + sampled; returns dict tier metrics."""
    res = {'greedy_compile': False, 'greedy_match': False, 'passk': False}
    _, g = kenchat.gen_tokens(prompt, weights, max_tokens=max_tokens,
                              temperature=None, codec=codec)
    full = prompt.rstrip() + '\n' + g.lstrip()
    res['greedy_compile'], res['greedy_match'], _ = run_candidate(full, want)
    res['passk'] = res['greedy_match']
    if not res['passk']:
        for i in range(k - 1):
            _, gs = kenchat.gen_tokens(prompt, weights, max_tokens=max_tokens,
                                       temperature=1.0, codec=codec, seed=i)
            f2 = prompt.rstrip() + '\n' + gs.lstrip()
            okc, okm, _ = run_candidate(f2, want)
            if okm:
                res['passk'] = True
                break
    return res


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--model', required=True)
    ap.add_argument('--max-tokens', type=int, default=220)
    ap.add_argument('-k', type=int, default=4)
    ap.add_argument('--frac', type=float, default=0.1)
    args = ap.parse_args()

    weights = f'minds/mid_prophet_{args.model}_w.txt'
    codec = kenchat.load_codec_vocab('minds/kenga_full.pkl')

    t1 = {'n': 0, 'compile': 0, 'match': 0, 'passk': 0}
    t2 = {'n': 0, 'compile': 0, 'run': 0}
    skipped = {'no-stdout': 0, 'fail': 0, 'long-main': 0, 'no-main': 0}

    for p in held_files(args.frac):
        name = norm(p).split('/')[-1]
        src = open(p, encoding='utf-8', errors='replace').read()
        rc0, out0, err0 = kenchat.run_via_kenga_lite(src, timeout=10)
        if rc0 != 0:
            skipped['fail'] += 1
            continue
        if not out0.strip():
            skipped['no-stdout'] += 1
            continue
        want = out0.strip().split('\n')[0]
        main_txt = extract_main(src)

        # ---- Tier 1: main-first controlled completion ----
        if main_txt is None:
            skipped['no-main'] += 1
        else:
            if len(kenchat.tokenize(main_txt, codec)) > 60:
                skipped['long-main'] += 1
            else:
                r = gen_and_score(main_txt, want, weights, codec,
                                  args.max_tokens, args.k)
                t1['n'] += 1
                t1['compile'] += int(r['greedy_compile'])
                t1['match'] += int(r['greedy_match'])
                t1['passk'] += int(r['passk'])
                print(f'T1 {name:26s} compile={r["greedy_compile"]} '
                      f'match={r["greedy_match"]} pass@k={r["passk"]} '
                      f'want={want}', flush=True)

        # ---- Tier 2: free continuation ----
        prompt2 = first_fn_block(src)
        if len(kenchat.tokenize(prompt2, codec)) <= 60:
            _, g2 = kenchat.gen_tokens(prompt2, weights,
                                       max_tokens=args.max_tokens,
                                       temperature=None, codec=codec)
            full2 = kenchat.make_valid_program(prompt2, g2)
            rc2, _, _ = kenchat.run_via_kenga_lite(full2, timeout=10)
            t2['n'] += 1
            t2['compile'] += int(rc2 == 0)
            t2['run'] += int(rc2 == 0)
            print(f'T2 {name:26s} compile={rc2 == 0}', flush=True)

    n1, n2 = t1['n'], t2['n']
    print(f'\nTIER 1 (controlled completion, main given): n={n1}')
    print(f'  compile {t1["compile"]}/{n1} ({100*t1["compile"]/max(1,n1):.0f}%)  '
          f'semantic match {t1["match"]}/{n1} ({100*t1["match"]/max(1,n1):.0f}%)  '
          f'match pass@{args.k} {t1["passk"]}/{n1} ({100*t1["passk"]/max(1,n1):.0f}%)')
    print(f'TIER 2 (free continuation): n={n2}')
    print(f'  valid-program rate {t2["run"]}/{n2} ({100*t2["run"]/max(1,n2):.0f}%)')
    print(f'skipped: {skipped}')
    return 0


if __name__ == '__main__':
    sys.exit(main())
