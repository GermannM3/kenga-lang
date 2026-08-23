"""tools/repair_eval.py — evaluate a model on Repair pairs.

For each held-out mutant: prompt = broken source; the model continues.
The continuation is sliced at top-level '}' boundaries; every candidate
(prompt + candidate-prefix) is compiled and run by kenga-lite. Success =
some candidate runs AND prints the fixed program's ground-truth stdout.

Metrics: fixed@1 (greedy), fixed@k (sampled).
"""
import argparse
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kenchat


def candidates(gen_src):
    """Yield generation prefixes ending at each top-level '}'."""
    depth = 0
    for i, ch in enumerate(gen_src):
        if ch == '{':
            depth += 1
        elif ch == '}':
            depth = max(0, depth - 1)
            if depth == 0:
                yield gen_src[:i + 1]


def try_fix(prompt, gen_src, want, codec, weights, max_tokens, temperature,
            seed, k_budget):
    tried = 0
    for cand in candidates(gen_src):
        full = prompt + '\n' + cand
        rc, out, _ = kenchat.run_via_kenga_lite(full, timeout=8)
        tried += 1
        if rc == 0 and out.strip().split('\n')[0] == want:
            return True, tried
        if k_budget and tried >= k_budget:
            break
    return False, tried


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--model', required=True)
    ap.add_argument('--eval', default='minds/repair_corpus/test_mutants.jsonl')
    ap.add_argument('--limit', type=int, default=100)
    ap.add_argument('--max-tokens', type=int, default=260)
    ap.add_argument('-k', type=int, default=4)
    args = ap.parse_args()

    weights = f'minds/mid_prophet_{args.model}_w.txt'
    codec = kenchat.load_codec_vocab('minds/kenga_full.pkl')

    recs = []
    for line in open(args.eval, encoding='utf-8'):
        line = line.strip()
        if line:
            recs.append(json.loads(line))
    recs = recs[:args.limit]

    n = ok1 = okk = 0
    modes = {}
    for r in recs:
        prompt = r['broken'].rstrip()
        _, g1 = kenchat.gen_tokens(prompt, weights, max_tokens=args.max_tokens,
                                   temperature=None, codec=codec)
        f1, _ = try_fix(prompt, g1, r['out'], codec, weights,
                        args.max_tokens, None, 0, None)
        fk = f1
        tries = 4
        if not fk:
            for i in range(args.k - 1):
                _, gs = kenchat.gen_tokens(prompt, weights,
                                           max_tokens=args.max_tokens,
                                           temperature=1.0, codec=codec, seed=i)
                ok, used = try_fix(prompt, gs, r['out'], codec, weights,
                                   args.max_tokens, 1.0, i, tries)
                if ok:
                    fk = True
                    break
        m = r['mode']
        modes.setdefault(m, [0, 0])
        modes[m][0] += int(f1)
        modes[m][1] += int(fk)
        ok1 += int(f1)
        okk += int(fk)
        n += 1

    print(f'repair eval: model={args.model} mutants={n} pass@{args.k}')
    for m, (a, b) in sorted(modes.items()):
        print(f'  {m:6s}: fixed@1 {a}/{n}, fixed@{args.k} {b}/{n}')
    print(f'TOTAL : fixed@1 {ok1}/{n} ({100*ok1/max(1,n):.1f}%)  '
          f'fixed@{args.k} {okk}/{n} ({100*okk/max(1,n):.1f}%)')
    return 0


if __name__ == '__main__':
    sys.exit(main())
