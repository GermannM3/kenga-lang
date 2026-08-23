"""tools/corpus_factory.py — Kenga Corpus Factory (Phase II: Data & Semantics).

Generates synthetic, compiler-verified Kenga programs. Every program is
executed via kenga-lite (compile -> run -> stdout); only rc==0 programs are
kept. Semantic-equivalent body variants are kept only when they reproduce
the exact same stdout. Token-level mutations of verified programs produce
(broken, fixed) repair pairs labelled by failure mode.

Categories: arith (expression functions), loop (accumulators), rec (single
self-recursion), chain (call chains).

Output: JSONL manifest, one record per program:
  {"id", "category", "src", "out",
   "variants": [{"src", "out"}],
   "mutants":  [{"src", "mode"}]}      mode: "run" | "value"
"""
import argparse
import json
import os
import random
import re
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kenchat


# ---------------------------------------------------------------- arith ----

def gen_arith(rng):
    """Expression functions with semantic-equivalent body variants."""
    npar = rng.choice([1, 1, 2, 2, 3])
    params = sorted(rng.sample(['a', 'b', 'c', 'd', 'e'], npar))

    def expr(d):
        if d <= 0 or rng.random() < 0.35:
            if params and rng.random() < 0.65:
                return rng.choice(params)
            return str(rng.randint(0, 9))
        op = rng.choice(['+', '-', '*', '*', '+'])
        s = f'{expr(d - 1)} {op} {expr(d - 1)}'
        return '(' + s + ')' if d >= 2 else s

    sig = ', '.join(f'{p}: i64' for p in params)
    callargs = ', '.join(str(rng.randint(0, 12)) for _ in params)
    e = expr(rng.randint(1, 3))

    bodies = [f'return {e};',
              f'let c = {e};\n    return c;']
    # top-level split: let t = L; t = t OP R; return t;
    # valid only when e is fully wrapped: "(L OP R)" with L,R paren-balanced
    flat = e.strip()
    if len(flat) > 4 and flat[0] == '(' and flat[-1] == ')':
        inner = flat[1:-1]
        depth = 0
        for i, ch in enumerate(inner):
            if ch == '(':
                depth += 1
            elif ch == ')':
                depth -= 1
            elif depth == 0 and ch in '+-*' and i > 0 and inner[i - 1] == ' ' \
                    and i + 1 < len(inner) and inner[i + 1] == ' ':
                l, r = inner[:i].strip(), inner[i + 1:].strip()
                if l.count('(') == l.count(')') and r.count('(') == r.count(')'):
                    op = ch
                    bodies.append(f'let t = {l};\n    t = t {op} {r};\n    return t;')
                break
    rng.shuffle(bodies)

    def mk(body):
        return (f'fn calc({sig}) -> i64 {{\n    {body}\n}}\n'
                f'fn main() -> i64 {{\n'
                f'    let r = calc({callargs});\n    println(r);\n    return 0;\n}}\n')

    return mk(bodies[0]), [mk(v) for v in bodies[1:3]]


# ----------------------------------------------------------------- loop ----

def gen_loop(rng):
    kind = rng.choice(['sum_range', 'count_range', 'prod_range'])
    if kind == 'sum_range':
        acc = rng.choice(['i', 'i', '2 * i', 'b - i'])
        step = rng.choice([1, 1, 2])
        body = (f'let s = 0;\n    let i = a;\n'
                f'    while i <= b {{\n        s = s + {acc};\n        i = i + {step};\n    }}\n'
                f'    return s;')
        var = (f'let s = 0;\n    let i = a;\n    while i <= b {{\n'
               f'        let t = {acc};\n        s = s + t;\n        i = i + {step};\n    }}\n    return s;')
        a, b = rng.randint(0, 14), rng.randint(0, 14)
    elif kind == 'count_range':
        acc = rng.choice(['i', 'i', 'a + b - i'])
        step = rng.choice([1, 1, 2])
        body = (f'let s = 0;\n    let i = b;\n'
                f'    while i >= a {{\n        s = s + {acc};\n        i = i - {step};\n    }}\n'
                f'    return s;')
        var = (f'let s = 0;\n    let i = b;\n    while i >= a {{\n'
               f'        let t = {acc};\n        s = s + t;\n        i = i - {step};\n    }}\n    return s;')
        a, b = rng.randint(0, 14), rng.randint(0, 14)
    else:
        a = rng.randint(0, 6)
        b = a + rng.randint(0, 6)
        body = (f'let p = 1;\n    let i = a;\n'
                f'    while i <= b {{\n        p = p * i;\n        i = i + 1;\n    }}\n'
                f'    return p;')
        var = None

    def mk(fnbody):
        return (f'fn run(a: i64, b: i64) -> i64 {{\n    {fnbody}\n}}\n'
                f'fn main() -> i64 {{\n'
                f'    let r = run({a}, {b});\n    println(r);\n    return 0;\n}}\n')

    variants = [mk(var)] if var else []
    return mk(body), variants


# ------------------------------------------------------------------ rec ----

def gen_rec(rng):
    shape = rng.choice(['fact', 'sumrec', 'halve', 'fib', 'sumrec1', 'halve1'])
    if shape == 'fact':
        base_cond, base_ret, op, dec = 1, 1, '*', 1
        n = rng.randint(0, 8)
        it = (f'let p = 1;\n    let i = 2;\n'
              f'    while i <= n {{\n        p = p * i;\n        i = i + 1;\n    }}\n'
              f'    return p;')
    elif shape == 'sumrec':
        base_cond, base_ret, op, dec = 1, 0, '+', 1
        n = rng.randint(0, 20)
        it = (f'let s = 0;\n    let i = 1;\n'
              f'    while i <= n {{\n        s = s + i;\n        i = i + 1;\n    }}\n'
              f'    return s;')
    elif shape == 'sumrec1':
        base_cond, base_ret, op, dec = 1, 1, '+', 1
        n = rng.randint(0, 20)
        it = (f'let s = 0;\n    let i = 2;\n'
              f'    while i <= n {{\n        s = s + i;\n        i = i + 1;\n    }}\n'
              f'    return s + 1;')
    elif shape == 'halve':
        base_cond, base_ret, op, dec = 2, 'n', '+', 2
        n = rng.randint(0, 16)
        it = (f'let s = 0;\n    let i = n;\n'
              f'    while i > 0 {{\n        s = s + i;\n        i = i - 2;\n    }}\n'
              f'    return s;')
    elif shape == 'halve1':
        base_cond, base_ret, op, dec = 2, 1, '+', 2
        n = rng.randint(0, 16)
        it = (f'let s = 0;\n    let i = n;\n'
              f'    while i > 1 {{\n        s = s + i;\n        i = i - 2;\n    }}\n'
              f'    return s + 1;')
    else:
        # classic fib: deep(n) = fib(n); iterative equivalent
        n = rng.randint(0, 16)
        rec_fn = (f'fn deep(n: i64) -> i64 {{\n'
                  f'    if n < 2 {{ return n; }}\n'
                  f'    return deep(n - 1) + deep(n - 2);\n}}')
        it_fn = (f'fn run(n: i64) -> i64 {{\n'
                 f'    let a = 0;\n    let b = 1;\n    let i = 0;\n'
                 f'    while i < n {{\n        let t = a + b;\n'
                 f'        a = b;\n        b = t;\n        i = i + 1;\n    }}\n'
                 f'    return a;\n}}')
        return (rec_fn + '\n' + it_fn + '\n'
                f'fn main() -> i64 {{\n    let r = deep({n});\n'
                f'    println(r);\n    return 0;\n}}\n',
                [rec_fn + '\n' + it_fn + '\n'
                 f'fn main() -> i64 {{\n    let r = run({n});\n'
                 f'    println(r);\n    return 0;\n}}\n'])

    rec_fn = (f'fn deep(n: i64) -> i64 {{\n'
              f'    if n < {base_cond} {{ return {base_ret}; }}\n'
              f'    return n {op} deep(n - {dec});\n}}')
    it_fn = f'fn run(n: i64) -> i64 {{\n    {it}\n}}'

    def mk(caller_body):
        return (rec_fn + '\n' + it_fn + '\n'
                f'fn main() -> i64 {{\n'
                f'    {caller_body}\n    return 0;\n}}\n')

    primary = mk(f'let r = deep({n});\n    println(r);')
    variant = mk(f'let r = run({n});\n    println(r);')
    return primary, [variant]


# ---------------------------------------------------------------- chain ----

def gen_chain(rng):
    depth = rng.randint(2, 4)
    consts = [rng.randint(0, 9) for _ in range(depth)]
    lines = []
    prev = 'x'
    for i in range(depth):
        fn = f'f{i}'
        if i == 0:
            body = f'return x + {consts[0]};'
        else:
            op = rng.choice(['+', '*'])
            rhs = str(consts[i]) if op == '+' else rng.choice(['2', '3'])
            body = f'return {prev} {op} {rhs};'
        lines.append(f'fn {fn}(x: i64) -> i64 {{\n    {body}\n}}')
        prev = fn
    arg = rng.randint(0, 9)
    lines.append(f'fn main() -> i64 {{\n    let r = {prev}({arg});\n'
                 f'    println(r);\n    return 0;\n}}')
    return '\n'.join(lines) + '\n', []


# ------------------------------------------------------------- mutations ---

OPFLIP = {' + ': ' - ', ' - ': ' + ', ' * ': ' + '}


def mutate(src, rng):
    """Return a broken copy of src, or None if nothing applicable."""
    kinds = []
    if any(o in src for o in OPFLIP):
        kinds.append('opflip')
    if re.search(r'\b\d+\b', src):
        kinds.append('bump')
    if ')' in src:
        kinds.append('droppar')
    m = re.search(r'fn (\w+)\(([^)]*)\)', src)
    if m and ',' in m.group(2):
        kinds.append('swapparams')
    if not kinds:
        return None
    k = rng.choice(kinds)
    if k == 'opflip':
        opts = [o for o in OPFLIP if o in src]
        o = rng.choice(opts)
        return src.replace(o, OPFLIP[o], 1)
    if k == 'bump':
        nums = list(re.finditer(r'\b\d+\b', src))
        mm = rng.choice(nums)
        v = int(mm.group(0))
        nv = v + 1 if rng.random() < 0.5 else max(0, v - 1)
        return src[:mm.start()] + str(nv) + src[mm.end():]
    if k == 'droppar':
        idxs = [i for i, ch in enumerate(src) if ch == ')']
        i = rng.choice(idxs)
        return src[:i] + src[i + 1:]
    mm = re.search(r'fn (\w+)\(([^)]*)\)', src)
    parts = [p.strip() for p in mm.group(2).split(',')]
    i, j = rng.sample(range(len(parts)), 2)
    ni, nj = parts[i].split(':')[0], parts[j].split(':')[0]
    parts[i] = parts[i].replace(ni, nj)
    parts[j] = parts[j].replace(nj, ni)
    return src[:mm.start(2)] + ', '.join(parts) + src[mm.end(2):]


# ------------------------------------------------------------------ main ---

GENS = {'arith': gen_arith, 'loop': gen_loop, 'rec': gen_rec, 'chain': gen_chain}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--arith', type=int, default=150)
    ap.add_argument('--loop', type=int, default=100)
    ap.add_argument('--rec', type=int, default=80)
    ap.add_argument('--chain', type=int, default=80)
    ap.add_argument('--max-mutants', type=int, default=1)
    ap.add_argument('--seed', type=int, default=13)
    ap.add_argument('--out', default='minds/corpus_factory/manifest.jsonl')
    args = ap.parse_args()

    rng = random.Random(args.seed)
    os.makedirs(os.path.dirname(args.out), exist_ok=True)

    counts = {'arith': args.arith, 'loop': args.loop,
              'rec': args.rec, 'chain': args.chain}
    seen = set()
    stats = {'kept': 0, 'dropped_primary': 0, 'variants_kept': 0,
             'variants_dropped': 0, 'mut_run': 0, 'mut_value': 0,
             'mut_neutral': 0}
    run_times = []

    t_start = time.time()
    n_lines = 0
    with open(args.out, 'w', encoding='utf-8') as f:
        for cat, want in counts.items():
            made = 0
            attempts = 0
            while made < want and attempts < want * 6:
                attempts += 1
                primary, variants = GENS[cat](rng)
                if primary in seen:
                    continue

                t0 = time.time()
                rc, out, err = kenchat.run_via_kenga_lite(primary, timeout=10)
                run_times.append(time.time() - t0)
                if rc != 0 or not out.strip():
                    stats['dropped_primary'] += 1
                    continue
                seen.add(primary)
                rec = {'id': f'{cat}_{made:05d}', 'category': cat,
                       'src': primary, 'out': out.strip(),
                       'variants': [], 'mutants': []}

                for vsrc in variants:
                    t0 = time.time()
                    vrc, vout, _ = kenchat.run_via_kenga_lite(vsrc, timeout=10)
                    run_times.append(time.time() - t0)
                    if vrc == 0 and vout.strip() == rec['out']:
                        rec['variants'].append({'src': vsrc, 'out': vout.strip()})
                        stats['variants_kept'] += 1
                    else:
                        stats['variants_dropped'] += 1

                for _ in range(args.max_mutants):
                    msrc = mutate(primary, rng)
                    if msrc is None:
                        continue
                    t0 = time.time()
                    # short timeout: a hanging mutant is a valid broken sample
                    mrc, mout, _ = kenchat.run_via_kenga_lite(msrc, timeout=4)
                    run_times.append(time.time() - t0)
                    if mrc != 0:
                        rec['mutants'].append({'src': msrc, 'mode': 'run'})
                        stats['mut_run'] += 1
                    elif mout.strip() != rec['out']:
                        rec['mutants'].append({'src': msrc, 'mode': 'value'})
                        stats['mut_value'] += 1
                    else:
                        stats['mut_neutral'] += 1

                f.write(json.dumps(rec) + '\n')
                n_lines += 1
                made += 1
                stats['kept'] += 1
                if made % 50 == 0:
                    print(f'  [{cat}] {made}/{want} kept, '
                          f'{len(run_times)} runs, {time.time()-t_start:.0f}s',
                          flush=True)

    wall = time.time() - t_start
    avg_ms = 1000 * sum(run_times) / max(1, len(run_times))
    print(f'manifest: {args.out}')
    print(f'programs kept: {stats["kept"]}  (dropped primary: {stats["dropped_primary"]})')
    print(f'per category (made/requested): ' +
          ', '.join(f'{c}={counts[c]}' for c in counts))
    print(f'variants: kept {stats["variants_kept"]}, dropped {stats["variants_dropped"]}')
    print(f'mutants: run-fail {stats["mut_run"]}, wrong-value {stats["mut_value"]}, '
          f'neutral-discarded {stats["mut_neutral"]}')
    print(f'kenga-lite runs: {len(run_times)}, avg {avg_ms:.0f} ms, wall {wall:.0f}s')
    print(f'estimated 100k programs: {avg_ms * 400000 / 1000 / 3600:.1f} h '
          f'(4 runs per program incl variants+mutants)')
    return 0


if __name__ == '__main__':
    sys.exit(main())
