"""tools/realgen_eval.py — generation eval on REAL held-out Kenga files.

For each held-out file that is small enough to be fully reproducible
(<= --max-tokens total tokens) and whose original run prints something:
  prompt  = first top-level fn block
  model   generates the rest (helpers + its own main)
  verify  = kenga-lite compile/run; semantic match = first stdout line
            equals the original file's first stdout line.

Metrics: compile%, run%, greedy match, pass@k. Zero-stdout files are
counted as compile-only (match n/a) and reported separately.
"""
import argparse
import hashlib
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kenchat
import train_m3


def held_files(frac=0.1):
    SKIP_BIG = {
        'bc_src_c.kenga','more.kenga','lower_kv.kenga','lower_c.kenga',
        'rt_prophet.kenga','native_ml.kenga','rt_vm.kenga','rt_tensor.kenga',
        'rt_kval_tape.kenga','rt_kval_mem.kenga',
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
                h = int(hashlib.md5(p.replace('\\', '/').encode()).hexdigest(), 16) % 10000
                if h < frac * 10000:
                    out.append(p)
    return sorted(out)


def first_fn_block(src):
    depth = 0
    for i, ch in enumerate(src):
        if ch == '{':
            depth += 1
        elif ch == '}':
            depth -= 1
            if depth == 0 and 'fn ' in src[:i]:
                return src[:i + 1]
    return src


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--model', required=True)
    ap.add_argument('--max-tokens', type=int, default=220)
    ap.add_argument('-k', type=int, default=4)
    args = ap.parse_args()

    weights = f'minds/mid_prophet_{args.model}_w.txt'
    codec = kenchat.load_codec_vocab('minds/kenga_full.pkl')

    cases = []
    skipped_long = skipped_nostdout = 0
    for p in held_files():
        src = open(p, encoding='utf-8', errors='replace').read()
        rc0, out0, _ = kenchat.run_via_kenga_lite(src, timeout=10)
        want = out0.strip().split('\n')[0] if out0.strip() else None
        if want is None or rc0 != 0:
            # try comments-stripped? no: file itself must run clean
            skipped_nostdout += 1
            continue
        prompt = first_fn_block(src)
        if len(kenchat.tokenize(prompt, codec)) > 60:
            skipped_long += 1
            continue
        cases.append((p, prompt, want))
    print(f'real-gen eval: {len(cases)} runnable small files '
          f'(skipped: {skipped_nostdout} no-stdout/fail, {skipped_long} long-prompt)')

    stats = {'compile': 0, 'run': 0, 'greedy': 0, 'passk': 0}
    detail = []
    for p, prompt, want in cases:
        name = p.replace('\\', '/').split('/')[-1]
        _, gsrc = kenchat.gen_tokens(prompt, weights, max_tokens=args.max_tokens,
                                     temperature=None, codec=codec)
        full = kenchat.make_valid_program(prompt, gsrc)
        rc, out, _ = kenchat.run_via_kenga_lite(full, timeout=10)
        first = out.strip().split('\n')[0] if out.strip() else ''
        ok_c = rc == 0
        ok_m = ok_c and first == want
        pk = ok_m
        if not pk:
            for i in range(args.k - 1):
                _, ssrc = kenchat.gen_tokens(prompt, weights,
                                             max_tokens=args.max_tokens,
                                             temperature=1.0, codec=codec, seed=i)
                sf = kenchat.make_valid_program(prompt, ssrc)
                src_rc, sout, _ = kenchat.run_via_kenga_lite(sf, timeout=10)
                sfirst = sout.strip().split('\n')[0] if sout.strip() else ''
                if src_rc == 0 and sfirst == want:
                    pk = True
                    break
        stats['compile'] += int(ok_c)
        stats['run'] += int(rc == 0)
        stats['greedy'] += int(ok_m)
        stats['passk'] += int(pk)
        detail.append((name, ok_c, rc == 0, ok_m, pk, want, first[:12]))
        print(f'  {name:28s} compile={ok_c} run={rc == 0} greedy={ok_m} '
              f'pass@k={pk} want={want} got={first[:12]}', flush=True)

    n = len(cases)
    print(f'\nTOTAL {n} files: compile {stats["compile"]}/{n} '
          f'({100*stats["compile"]/max(1,n):.0f}%)  '
          f'run {stats["run"]}/{n} ({100*stats["run"]/max(1,n):.0f}%)  '
          f'greedy-match {stats["greedy"]}/{n} ({100*stats["greedy"]/max(1,n):.0f}%)  '
          f'pass@{args.k} {stats["passk"]}/{n} ({100*stats["passk"]/max(1,n):.0f}%)')
    return 0


if __name__ == '__main__':
    sys.exit(main())
