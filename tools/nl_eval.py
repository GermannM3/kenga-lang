"""tools/nl_eval.py — NL->code eval on Factory v3 test split.

Prompt = ONLY the task comment line (the spec). The model must produce the
whole program. Verifier: kenga-lite compile+run; match = stdout equals the
record's ground truth (fair here: factory constants live inside the spec).

Usage:
  python tools/nl_eval.py --model m6 --test minds/corpus_factory/split_v3/test.jsonl
"""
import argparse
import collections
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kenchat

KEEP = os.environ.get('NL_KEEP_COMMENTS','1')=='1'


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--model', required=True)
    ap.add_argument('--codec', default='minds/kenga_full.pkl')
    ap.add_argument('--test', default='minds/corpus_factory/split_v3/test.jsonl')
    ap.add_argument('--limit', type=int, default=100)
    ap.add_argument('--max-tokens', type=int, default=200)
    ap.add_argument('-k', type=int, default=4)
    args = ap.parse_args()

    weights = f'minds/mid_prophet_{args.model}_w.txt'
    codec = kenchat.load_codec_vocab(args.codec)

    recs = []
    for line in open(args.test, encoding='utf-8'):
        line = line.strip()
        if line:
            r = json.loads(line)
            if r['src'].startswith('//'):
                recs.append(r)
    recs = recs[:args.limit]

    st = collections.defaultdict(lambda: [0, 0, 0])
    n = 0
    import os as _os
    done = set()
    if _os.path.exists(args.test + '.nl_progress'):
        for l in open(args.test + '.nl_progress', encoding='utf-8'):
            if l.startswith('DONE '):
                done.add(l.split()[1])
    prog = open(args.test + '.nl_progress', 'a', encoding='utf-8')
    for r in recs:
        if r['id'] in done:
            continue
        prompt = r['src'].split('\n')[0]  # comment line only
        _, g = kenchat.gen_tokens(prompt, weights, max_tokens=args.max_tokens,
                                  temperature=None, codec=codec,
                                  keep_comments=KEEP)
        full = kenchat.make_valid_program(prompt + '\n', g)
        rc, out, _ = kenchat.run_via_kenga_lite(full, timeout=10)
        first = out.strip().split('\n')[0] if out.strip() else ''
        okc = rc == 0
        okm = okc and first == r['out']
        pk = okm
        if not pk:
            for i in range(args.k - 1):
                _, gs = kenchat.gen_tokens(prompt, weights,
                                           max_tokens=args.max_tokens,
                                           temperature=1.0, codec=codec, seed=i,
                                           keep_comments=KEEP)
                f2 = kenchat.make_valid_program(prompt + '\n', gs)
                rc2, out2, _ = kenchat.run_via_kenga_lite(f2, timeout=10)
                f2o = out2.strip().split('\n')[0] if out2.strip() else ''
                if rc2 == 0 and f2o == r['out']:
                    pk = True
                    break
        s = st[r['category']]
        s[0] += int(okc); s[1] += int(okm); s[2] += int(pk)
        n += 1
        prog.write("DONE " + r["id"] + "\n"); prog.flush()
        print(f'{r["id"]} compile={okc} match={okm} pass@k={pk}', flush=True)

    tot = [0, 0, 0]
    print(f'NL->code: model={args.model} programs={n} pass@{args.k}')
    print(f'{"category":10s} {"compile":>10s} {"match":>10s} {"pass@k":>10s}')
    for cat in sorted(st):
        c, m_, pk = st[cat]
        tot[0] += c; tot[1] += m_; tot[2] += pk
        print(f'{cat:10s} {c:>5d}/{n:<3d} {m_:>5d}/{n:<3d} {pk:>5d}/{n:<3d}')
    print(f'{"TOTAL":10s} {tot[0]:>5d}/{n:<3d} {tot[1]:>5d}/{n:<3d} {tot[2]:>5d}/{n:<3d}')
    print(f'\ncompile {100*tot[0]/max(1,n):.1f}%  '
          f'match(greedy) {100*tot[1]/max(1,n):.1f}%  '
          f'match(pass@{args.k}) {100*tot[2]/max(1,n):.1f}%')
    return 0


if __name__ == '__main__':
    sys.exit(main())
