"""tools/corpus_split.py — Corpus Factory statistics + template-aware split.

Directive (Phase II): split train/test BY TEMPLATE, not by random rows.
A template is the program source with all integer literals masked to '#',
so two programs differing only in constants share a template. Whole
template groups go to one side; equivalence variants follow their primary.

Outputs:
  <out>/train.jsonl
  <out>/test.jsonl
and prints corpus statistics (categories, token lengths, uniqueness,
equivalence share, mutant modes, template counts, split sizes).
"""
import argparse
import collections
import json
import os
import random
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kenchat


def mask_literals(src):
    return re.sub(r'\b\d+\b', '#', src)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('manifest')
    ap.add_argument('--test-frac', type=float, default=0.1)
    ap.add_argument('--seed', type=int, default=13)
    ap.add_argument('--out', default='minds/corpus_factory/split')
    args = ap.parse_args()

    recs = []
    with open(args.manifest, encoding='utf-8') as f:
        for line in f:
            line = line.strip()
            if line:
                recs.append(json.loads(line))

    codec = kenchat.load_codec_vocab('minds/kenga_full.pkl')

    # ---- statistics ----
    per_cat = collections.Counter(r['category'] for r in recs)
    lens = []
    for r in recs:
        lens.append(len(kenchat.tokenize(r['src'], codec)))
    lens_sorted = sorted(lens)
    n = len(lens_sorted)
    med = lens_sorted[n // 2]
    p90 = lens_sorted[int(n * 0.9)]
    uniq = len(set(r['src'] for r in recs))
    with_var = sum(1 for r in recs if r['variants'])
    n_var = sum(len(r['variants']) for r in recs)
    mut = collections.Counter(m['mode'] for r in recs for m in r['mutants'])

    templates = collections.defaultdict(list)
    for r in recs:
        templates[(r['category'], mask_literals(r['src']))].append(r)

    print(f'manifest: {args.manifest}')
    print(f'programs: {len(recs)}  unique sources: {uniq}')
    print(f'per category: ' + ', '.join(f'{c}={per_cat[c]}' for c in sorted(per_cat)))
    print(f'token length: min={lens_sorted[0]} median={med} p90={p90} max={lens_sorted[-1]}')
    print(f'equivalence: {with_var} programs with variants ({100*with_var/len(recs):.1f}%), '
          f'{n_var} variants total')
    print(f'mutants: ' + ', '.join(f'{k}={v}' for k, v in sorted(mut.items())))
    print(f'templates: {len(templates)} '
          f'(avg {len(recs)/len(templates):.1f} programs per template)')

    # ---- template-aware split ----
    rng = random.Random(args.seed)
    train, test = [], []
    by_cat = collections.defaultdict(list)
    for key, group in templates.items():
        by_cat[key[0]].append(group)
    for cat, groups in sorted(by_cat.items()):
        rng.shuffle(groups)
        k_test = max(1, int(len(groups) * args.test_frac))
        for g in groups[:k_test]:
            test.extend(g)
        for g in groups[k_test:]:
            train.extend(g)

    # leakage check: no template on both sides
    tr_t = set((r['category'], mask_literals(r['src'])) for r in train)
    te_t = set((r['category'], mask_literals(r['src'])) for r in test)
    overlap = tr_t & te_t

    os.makedirs(args.out, exist_ok=True)
    for name, part in (('train', train), ('test', test)):
        with open(os.path.join(args.out, f'{name}.jsonl'), 'w', encoding='utf-8') as f:
            for r in part:
                f.write(json.dumps(r) + '\n')

    tr_cat = collections.Counter(r['category'] for r in train)
    te_cat = collections.Counter(r['category'] for r in test)
    print(f'\nsplit: train={len(train)} ({len(tr_t)} templates), '
          f'test={len(test)} ({len(te_t)} templates)')
    print(f'  train: ' + ', '.join(f'{c}={tr_cat[c]}' for c in sorted(tr_cat)))
    print(f'  test:  ' + ', '.join(f'{c}={te_cat[c]}' for c in sorted(te_cat)))
    print(f'template overlap between train/test: {len(overlap)} (must be 0)')
    return 0 if not overlap else 1


if __name__ == '__main__':
    sys.exit(main())
