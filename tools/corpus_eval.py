"""tools/corpus_eval.py — generation eval on Corpus Factory test split.

For each test record, prompt = first function block only; the model must
generate the rest (helper functions + main with a correct call). Programs
are verified by kenga-lite; stdout must equal the record's ground truth.

Metrics per directive point 8: compile-ok, run-ok, greedy match, pass@k.

Usage:
  python tools/corpus_eval.py --model m42 --test minds/corpus_factory/split/test.jsonl
"""
import argparse
import collections
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kenchat

CODECS = {
    'm37': 'minds/kenga_digits.pkl',
    'm40': 'minds/kenga_full.pkl',
    'm41': 'minds/kenga_full.pkl',
    'm42': 'minds/kenga_full.pkl',
    'm5': 'minds/kenga_full.pkl',
    'm52': 'minds/kenga_full.pkl',
    'm53': 'minds/kenga_full.pkl',
}
WEIGHTS = {m: f'minds/mid_prophet_{m}_w.txt' for m in CODECS}


def first_fn_block(src):
    """Prompt prefix: source up to and including the first top-level '}'.'
    The model must continue with helper functions and main."""
    depth = 0
    for i, ch in enumerate(src):
        if ch == '{':
            depth += 1
        elif ch == '}':
            depth -= 1
            if depth == 0:
                return src[:i + 1]
    return src


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--model', required=True, choices=list(CODECS))
    ap.add_argument('--test', default='minds/corpus_factory/split/test.jsonl')
    ap.add_argument('--limit', type=int, default=100)
    ap.add_argument('--category', default=None,
                    help='restrict to one category (e.g. bind for binding-test)')
    ap.add_argument('--max-tokens', type=int, default=200)
    ap.add_argument('-k', type=int, default=8, help='pass@k sampling budget')
    args = ap.parse_args()

    weights = WEIGHTS[args.model]
    codec = kenchat.load_codec_vocab(CODECS[args.model])

    recs = []
    with open(args.test, encoding='utf-8') as f:
        for line in f:
            line = line.strip()
            if line:
                recs.append(json.loads(line))
    if args.category:
        recs = [r for r in recs if r['category'] == args.category]
    recs = recs[:args.limit]

    stats = collections.defaultdict(lambda: [0, 0, 0, 0])  # compile,run,match,passk
    n = 0
    for r in recs:
        prompt = first_fn_block(r['src'])
        # greedy
        _, gsrc = kenchat.gen_tokens(prompt, weights, max_tokens=args.max_tokens,
                                     temperature=None, codec=codec)
        full = kenchat.make_valid_program(prompt, gsrc)
        rc, out, _ = kenchat.run_via_kenga_lite(full, timeout=10)
        first = out.strip().split('\n')[0] if out else ''
        ok_compile = rc == 0
        ok_match = first == r['out']
        passed_k = ok_match
        # pass@k: sampled candidates until match
        if not passed_k and args.k > 1:
            for i in range(args.k - 1):
                _, ssrc = kenchat.gen_tokens(prompt, weights,
                                             max_tokens=args.max_tokens,
                                             temperature=1.0, codec=codec, seed=i)
                ffull = kenchat.make_valid_program(prompt, ssrc)
                frc, fout, _ = kenchat.run_via_kenga_lite(ffull, timeout=10)
                ffirst = fout.strip().split('\n')[0] if fout else ''
                if frc == 0 and ffirst == r['out']:
                    passed_k = True
                    break
        st = stats[r['category']]
        st[0] += int(ok_compile)
        st[1] += int(ok_compile and rc == 0)
        st[2] += int(ok_match)
        st[3] += int(passed_k)
        n += 1

    tot = [0, 0, 0, 0]
    print(f'model={args.model} test={args.test} programs={n} pass@{args.k}')
    print(f'{"category":10s} {"compile":>10s} {"run":>10s} {"match":>10s} {"pass@k":>10s}')
    for cat in sorted(stats):
        c, ru, m_, pk = stats[cat]
        tot[0] += c; tot[1] += ru; tot[2] += m_; tot[3] += pk
        print(f'{cat:10s} {c:>6d}/{n:<3d} {ru:>6d}/{n:<3d} '
              f'{m_:>6d}/{n:<3d} {pk:>6d}/{n:<3d}')
    print(f'{"TOTAL":10s} {tot[0]:>6d}/{n:<3d} {tot[1]:>6d}/{n:<3d} '
          f'{tot[2]:>6d}/{n:<3d} {tot[3]:>6d}/{n:<3d}')
    pct = lambda x: f'{100*x/n:.1f}%'
    print(f'\ncompile {pct(tot[0])}  run {pct(tot[1])}  '
          f'match(greedy) {pct(tot[2])}  match(pass@{args.k}) {pct(tot[3])}')
    return 0


if __name__ == '__main__':
    sys.exit(main())
