"""tools/build_repair_corpus.py — build the Repair Model training corpus.

Reads factory manifest(s), emits one .kenga file per (broken, fixed) pair:
    <broken source>\\n<fixed source>
The causal LM objective on this document teaches: "after a broken program,
emit its corrected version". Train pairs come from the train split only;
test-split mutants are exported separately for evaluation.
"""
import json
import os
import sys


def main():
    import argparse
    ap = argparse.ArgumentParser()
    ap.add_argument('--train', default='minds/corpus_factory/split_v2/train.jsonl')
    ap.add_argument('--test', default='minds/corpus_factory/split_v2/test.jsonl')
    ap.add_argument('--out', default='minds/repair_corpus')
    ap.add_argument('--marker', default='',
                    help="embed a task-boundary marker line between broken and fixed (e.g. FIX)")
    args = ap.parse_args()
    outdir = args.out
    os.makedirs(outdir, exist_ok=True)

    n_docs = 0
    modes = {'run': 0, 'value': 0}
    for line in open(args.train, encoding='utf-8'):
        line = line.strip()
        if not line:
            continue
        rec = json.loads(line)
        for j, m in enumerate(rec.get('mutants', [])):
            sep = f'\n{args.marker}\n' if args.marker else '\n'
            doc = m['src'].rstrip() + sep + rec['src']
            path = os.path.join(outdir, f'pair_{n_docs:05d}.kenga')
            open(path, 'w', encoding='utf-8').write(doc)
            n_docs += 1
            modes[m['mode']] = modes.get(m['mode'], 0) + 1

    # eval set: mutants of template-disjoint test records
    eval_recs = []
    for line in open(args.test, encoding='utf-8'):
        line = line.strip()
        if not line:
            continue
        rec = json.loads(line)
        for m in rec.get('mutants', []):
            eval_recs.append({'broken': m['src'], 'fixed': rec['src'],
                              'out': rec['out'], 'mode': m['mode']})
    with open('minds/repair_test_mutants.jsonl', 'w', encoding='utf-8') as f:
        for r in eval_recs:
            f.write(json.dumps(r) + '\n')

    print(f'repair corpus: {n_docs} docs -> {outdir}/')
    print(f'modes: {modes}')
    print(f'eval mutants: {len(eval_recs)} -> minds/repair_test_mutants.jsonl')
    return 0


if __name__ == '__main__':
    sys.exit(main())
