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


# ---------------------------------------------------------------- pools ----
# M5.3 (Semantic Binding / Factory v2): identifiers must not be statistical
# shortcuts. Function/local names are sampled per program; distractor
# functions with matching signatures force true call-site binding.
FN_POOL = ['run', 'calc', 'compute', 'solve', 'apply', 'process', 'work',
           'transform', 'value_of', 'sum_up', 'mul_all', 'step', 'advance',
           'measure', 'scale', 'shift', 'fact', 'accumulate', 'fold',
           'reduce', 'build', 'derive', 'adjust', 'blend', 'eval_x', 'conv']
LOCAL_POOL = ['r', 's', 't', 'u', 'v', 'w', 'res', 'out', 'tmp', 'val',
              'acc', 'total', 'cur', 'num']


def pick(rng, pool):
    return rng.choice(pool)


def pick_distinct(rng, pool, n):
    out = rng.sample(pool, n)
    return out


# ------------------------------------------------------------------- nl ----
# M6 (NL->code): every program carries a task comment. The comment IS the
# spec — tokenizer keeps it when M3_KEEP_COMMENTS=1, so the model learns
# description -> implementation.

def nl_for(rng, kind, **kw):
    T = {
        'arith': ['// task: compute {e}',
                  '// calculate this expression of {p} numbers',
                  '// evaluate the formula over the arguments',
                  '// math helper: combine inputs and constants'],
        'loop': ['// task: sum values from a to b',
                 '// accumulate over the range with a while loop',
                 '// walk indices and add them into an accumulator',
                 '// total up every step across [a, b]'],
        'rec': ['// task: recursive {shape} over n, plus its loop form',
                '// self-recursive function and an equivalent iteration',
                '// recursion with base case; loop version included',
                '// define deep recursion and run same by while'],
        'chain': ['// task: chain of calls f0 then next until main prints',
                  "// pipeline: each function feeds the next; main prints last",
                  '// compose helpers so main output flows through all of them',
                  '// call chain ending in println of final value'],
        'bind': ['// task: main must call the right function by name',
                 '// several functions share the signature; use the correct one',
                 '// pick the intended definition for the call inside main',
                 '// binding drill: ignore decoys, call target only'],
        'str_build': ['// task: build a string of {n} letters by loop',
                      '// accumulate characters into a string n times',
                      '// grow a text buffer inside a while loop to length {n}',
                      '// repeat one symbol n times into a string'],
        'str_misc': ['// task: string length and comparison drill',
                     '// measure text and compare against another literal',
                     '// report size of a word or check if equal',
                     '// len and equality check on short strings'],
        'list_sum': ['// task: sum all elements of the list by index',
                     '// add every number stored in the list',
                     '// walk positions 0..len-1 accumulating xs[i]',
                     '// reduce list contents into one total'],
        'list_get': ['// task: read one element of the list by index',
                     '// fetch position k from fixed data and print it',
                     '// index access drill on a constant list',
                     '// take single item out of the array'],
    }[kind]
    s = rng.choice(T)
    try:
        return s.format(**kw)
    except KeyError:
        return s


# ---------------------------------------------------------------- arith ----

def gen_arith(rng):
    """Expression functions with semantic-equivalent body variants."""
    npar = rng.choice([1, 1, 2, 2, 3])
    params = sorted(rng.sample(['a', 'b', 'c', 'd', 'e'], npar))
    fname = pick(rng, FN_POOL)

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

    l1, l2 = pick_distinct(rng, LOCAL_POOL, 2)
    bodies = [f'return {e};',
              f'let {l1} = {e};\n    return {l1};']
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
                    bodies.append(f'let {l2} = {l};\n    {l2} = {l2} {op} {r};\n    return {l2};')
                break
    rng.shuffle(bodies)

    def mk(body):
        hdr = nl_for(rng, 'arith', p=npar)
        return (hdr + '\n' + f'fn {fname}({sig}) -> i64 {{\n    {body}\n}}\n'
                f'fn main() -> i64 {{\n'
                f'    let r = {fname}({callargs});\n    println(r);\n    return 0;\n}}\n')

    return mk(bodies[0]), [mk(v) for v in bodies[1:3]]


# ----------------------------------------------------------------- loop ----

def gen_loop(rng):
    kind = rng.choice(['sum_range', 'count_range', 'prod_range'])
    fname = pick(rng, FN_POOL)
    la, lb, ls = pick_distinct(rng, LOCAL_POOL, 3)
    if kind == 'sum_range':
        acc = rng.choice(['i', 'i', '2 * i', f'{lb} - i'])
        step = rng.choice([1, 1, 2])
        body = (f'let {ls} = 0;\n    let i = a;\n'
                f'    while i <= b {{\n        {ls} = {ls} + {acc};\n        i = i + {step};\n    }}\n'
                f'    return {ls};')
        var = (f'let {ls} = 0;\n    let i = a;\n    while i <= b {{\n'
               f'        let {la} = {acc};\n        {ls} = {ls} + {la};\n        i = i + {step};\n    }}\n    return {ls};')
        a, b = rng.randint(0, 14), rng.randint(0, 14)
    elif kind == 'count_range':
        acc = rng.choice(['i', 'i', f'a + b - i'])
        step = rng.choice([1, 1, 2])
        body = (f'let {ls} = 0;\n    let i = b;\n'
                f'    while i >= a {{\n        {ls} = {ls} + {acc};\n        i = i - {step};\n    }}\n'
                f'    return {ls};')
        var = (f'let {ls} = 0;\n    let i = b;\n    while i >= a {{\n'
               f'        let {la} = {acc};\n        {ls} = {ls} + {la};\n        i = i - {step};\n    }}\n    return {ls};')
        a, b = rng.randint(0, 14), rng.randint(0, 14)
    else:
        a = rng.randint(0, 6)
        b = a + rng.randint(0, 6)
        body = (f'let {ls} = 1;\n    let i = a;\n'
                f'    while i <= b {{\n        {ls} = {ls} * i;\n        i = i + 1;\n    }}\n'
                f'    return {ls};')
        var = None

    def mk(fnbody):
        hdr = nl_for(rng, 'loop')
        return (hdr + '\n' + f'fn {fname}(a: i64, b: i64) -> i64 {{\n    {fnbody}\n}}\n'
                f'fn main() -> i64 {{\n'
                f'    let r = {fname}({a}, {b});\n    println(r);\n    return 0;\n}}\n')

    variants = [mk(var)] if var else []
    return mk(body), variants


# ------------------------------------------------------------------ rec ----

def gen_rec(rng):
    shape = rng.choice(['fact', 'sumrec', 'halve', 'fib', 'sumrec1', 'halve1'])
    rname = pick(rng, FN_POOL)
    iname = pick_distinct(rng, [n for n in FN_POOL], 1)[0]
    if iname == rname:
        iname = iname + '_it'
    ls = pick(rng, LOCAL_POOL)
    la = pick_distinct(rng, [x for x in LOCAL_POOL if x != ls], 1)[0]
    lb = pick_distinct(rng, [x for x in LOCAL_POOL if x not in (ls, la)], 1)[0]
    if shape == 'fact':
        base_cond, base_ret, op, dec = 1, 1, '*', 1
        n = rng.randint(0, 8)
        it = (f'let {ls} = 1;\n    let i = 2;\n'
              f'    while i <= n {{\n        {ls} = {ls} * i;\n        i = i + 1;\n    }}\n'
              f'    return {ls};')
    elif shape == 'sumrec':
        base_cond, base_ret, op, dec = 1, 0, '+', 1
        n = rng.randint(0, 20)
        it = (f'let {ls} = 0;\n    let i = 1;\n'
              f'    while i <= n {{\n        {ls} = {ls} + i;\n        i = i + 1;\n    }}\n'
              f'    return {ls};')
    elif shape == 'sumrec1':
        base_cond, base_ret, op, dec = 1, 1, '+', 1
        n = rng.randint(0, 20)
        it = (f'let {ls} = 0;\n    let i = 2;\n'
              f'    while i <= n {{\n        {ls} = {ls} + i;\n        i = i + 1;\n    }}\n'
              f'    return {ls} + 1;')
    elif shape == 'halve':
        base_cond, base_ret, op, dec = 2, 'n', '+', 2
        n = rng.randint(0, 16)
        it = (f'let {ls} = 0;\n    let i = n;\n'
              f'    while i > 0 {{\n        {ls} = {ls} + i;\n        i = i - 2;\n    }}\n'
              f'    return {ls};')
    elif shape == 'halve1':
        base_cond, base_ret, op, dec = 2, 1, '+', 2
        n = rng.randint(0, 16)
        it = (f'let {ls} = 0;\n    let i = n;\n'
              f'    while i > 1 {{\n        {ls} = {ls} + i;\n        i = i - 2;\n    }}\n'
              f'    return {ls} + 1;')
    else:
        # classic fib: deep(n) = fib(n); iterative equivalent
        n = rng.randint(0, 16)
        rec_fn = (f'fn {rname}(n: i64) -> i64 {{\n'
                  f'    if n < 2 {{ return n; }}\n'
                  f'    return {rname}(n - 1) + {rname}(n - 2);\n}}')
        it_fn = (f'fn {iname}(n: i64) -> i64 {{\n'
                 f'    let {la} = 0;\n    let {lb} = 1;\n    let i = 0;\n'
                 f'    while i < n {{\n        let t = {la} + {lb};\n'
                 f'        {la} = {lb};\n        {lb} = t;\n        i = i + 1;\n    }}\n'
                 f'    return {la};\n}}')
        return (rec_fn + '\n' + it_fn + '\n'
                f'fn main() -> i64 {{\n    let r = {rname}({n});\n'
                f'    println(r);\n    return 0;\n}}\n',
                [rec_fn + '\n' + it_fn + '\n'
                 f'fn main() -> i64 {{\n    let r = {iname}({n});\n'
                 f'    println(r);\n    return 0;\n}}\n'])

    rec_fn = (f'fn {rname}(n: i64) -> i64 {{\n'
              f'    if n < {base_cond} {{ return {base_ret}; }}\n'
              f'    return n {op} {rname}(n - {dec});\n}}')
    it_fn = f'fn {iname}(n: i64) -> i64 {{\n    {it}\n}}'

    def mk(caller_body):
        hdr = nl_for(rng, 'rec', shape=shape)
        return (hdr + '\n' + rec_fn + '\n' + it_fn + '\n'
                f'fn main() -> i64 {{\n'
                f'    {caller_body}\n    return 0;\n}}\n')

    primary = mk(f'let r = {rname}({n});\n    println(r);')
    variant = mk(f'let r = {iname}({n});\n    println(r);')
    return primary, [variant]


# ---------------------------------------------------------------- chain ----

def gen_chain(rng):
    depth = rng.choice([2, 2, 3, 3, 4])
    names = pick_distinct(rng, FN_POOL, depth + rng.choice([0, 0, 1]))
    consts = [rng.randint(0, 9) for _ in range(depth)]
    lines = []
    prev = 'x'
    for i in range(depth):
        fn = names[i]
        if i == 0:
            body = f'return x + {consts[0]};'
        else:
            op = rng.choice(['+', '*'])
            rhs = str(consts[i]) if op == '+' else rng.choice(['2', '3'])
            body = f'return {prev} {op} {rhs};'
        lines.append(f'fn {fn}(x: i64) -> i64 {{\n    {body}\n}}')
        prev = fn
    # distractor: same arity as top, never called by main
    if len(names) > depth:
        d = names[depth]
        dc = rng.randint(1, 9)
        dop = rng.choice(['-', '*'])
        lines.insert(rng.randrange(len(lines)),
                     f'fn {d}(x: i64) -> i64 {{\n    return x {dop} {dc};\n}}')
    arg = rng.randint(0, 9)
    lr = pick(rng, LOCAL_POOL)
    lines.append(f'fn main() -> i64 {{\n    let {lr} = {prev}({arg});\n'
                 f'    println({lr});\n    return 0;\n}}')
    hdr = nl_for(rng, 'chain') + '\n'
    return hdr + '\n'.join(lines) + '\n', []


# ----------------------------------------------------------------- bind ----

def gen_bind(rng):
    """Binding drill: target fn + same-signature distractors; main must call
    the TARGET (name is the only discriminator)."""
    npar = rng.choice([1, 1, 1, 2])
    params = sorted(rng.sample(['a', 'b', 'c'], npar))
    sig = ', '.join(f'{p}: i64' for p in params)
    target = pick(rng, FN_POOL)
    ndis = rng.choice([1, 1, 2])
    dis = pick_distinct(rng, [n for n in FN_POOL if n != target], ndis)

    argvals = [str(rng.randint(0, 12)) for _ in params]

    used = set()

    def mkfn(name):
        while True:
            op = rng.choice(['+', '-', '*', '+'])
            c = rng.randint(1, 9)
            if (op, c) not in used:
                used.add((op, c))
                break
        return f'fn {name}({sig}) -> i64 {{\n    return {params[0]} {op} {c};\n}}'

    defs = [(target, mkfn(target))] + [(d, mkfn(d)) for d in dis]
    rng.shuffle(defs)
    text_defs = '\n'.join(x[1] for x in defs)
    args = ', '.join(argvals)
    l1, l2 = pick_distinct(rng, LOCAL_POOL, 2)
    pat = rng.choice(['let', 'let2', 'chained'])
    if pat == 'let':
        main_body = (f'let {l1} = {target}({args});\n'
                     f'    println({l1});')
    elif pat == 'let2':
        main_body = (f'let {l1} = {target}({args});\n'
                     f'    let {l2} = {l1};\n    println({l2});')
    else:
        main_body = (f'let {l1} = {target}({args});\n'
                     f'    let {l2} = {l1} + 0;\n    println({l2});')
    src = (nl_for(rng, 'bind') + '\n' + text_defs + '\n'
           f'fn main() -> i64 {{\n    {main_body}\n    return 0;\n}}\n')
    return src, []


# ------------------------------------------------------------ str/list ----

def gen_str_build(rng):
    n = rng.randint(1, 12)
    ch = rng.choice(['x', 'o', '#', '*', 'a'])
    fname = pick(rng, FN_POOL)
    ls, li = pick_distinct(rng, LOCAL_POOL, 2)
    body = (f'let {ls}: str = "";\n'
            f'    let {li} = 0;\n'
            f'    while {li} < n {{\n        {ls} = {ls} + "{ch}";\n        {li} = {li} + 1;\n    }}\n'
            f'    return {ls};')
    hdr = nl_for(rng, 'str_build', n=n)
    return (hdr + f'\nfn {fname}(n: i64) -> str {{\n    {body}\n}}\n'
            f'fn main() -> i64 {{\n'
            f'    let s: str = {fname}({n});\n'
            f'    println(len(s));\n    return 0;\n}}\n'), []


def gen_str_misc(rng):
    kind = rng.choice(['len', 'cmp', 'index'])
    w = rng.choice(['cat', 'dog', 'sun', 'box', 'red', 'tea'])
    other = rng.choice([w] + ['moon', 'sky', 'jar'])
    hdr = nl_for(rng, 'str_misc') + '\n'
    if kind == 'len':
        core = (f'fn main() -> i64 {{\n    let w: str = "{w}";\n'
                f'    println(len(w));\n    return 0;\n}}\n')
    elif kind == 'cmp':
        eq = other == w
        core = (f'fn main() -> i64 {{\n    let a: str = "{w}";\n'
                f'    let b: str = "{other}";\n'
                f'    if a == b {{ println(1); }} else {{ println(0); }}\n'
                f'    return 0;\n}}\n')
    else:
        k = rng.randint(0, len(w) - 1)
        core = (f'fn main() -> i64 {{\n    let w: str = "{w}";\n'
                f'    println(w[{k}]);\n    return 0;\n}}\n')
    return hdr + core, []


def gen_list_sum(rng):
    xs = [rng.randint(-9, 9) for _ in range(rng.randint(3, 6))]
    lit = ', '.join(str(v) for v in xs)
    fname = pick(rng, FN_POOL)
    ls, li = pick_distinct(rng, LOCAL_POOL, 2)
    hdr = nl_for(rng, 'list_sum')
    src = (hdr + f'\nfn {fname}(xs: list) -> i64 {{\n'
           f'    let {ls} = 0;\n    let {li} = 0;\n'
           f'    while {li} < len(xs) {{\n'
           f'        {ls} = {ls} + xs[{li}];\n        {li} = {li} + 1;\n    }}\n'
           f'    return {ls};\n}}\n'
           f'fn main() -> i64 {{\n    let data: list = [{lit}];\n'
           f'    println({fname}(data));\n    return 0;\n}}\n')
    return src, []


def gen_list_get(rng):
    xs = [rng.randint(-9, 9) for _ in range(rng.randint(3, 7))]
    k = rng.randrange(len(xs))
    lit = ', '.join(str(v) for v in xs)
    fname = pick(rng, FN_POOL)
    lr = pick(rng, LOCAL_POOL)
    hdr = nl_for(rng, 'list_get')
    src = (hdr + f'\nfn {fname}(xs: list, k: i64) -> i64 {{\n'
           f'    return xs[k];\n}}\n'
           f'fn main() -> i64 {{\n    let data: list = [{lit}];\n'
           f'    let {lr} = {fname}(data, {k});\n'
           f'    println({lr});\n    return 0;\n}}\n')
    return src, []


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

GENS = {'arith': gen_arith, 'loop': gen_loop, 'rec': gen_rec,
        'chain': gen_chain, 'bind': gen_bind, 'str_build': gen_str_build,
        'str_misc': gen_str_misc, 'list_sum': gen_list_sum,
        'list_get': gen_list_get}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--arith', type=int, default=6000)
    ap.add_argument('--loop', type=int, default=1800)
    ap.add_argument('--rec', type=int, default=400)
    ap.add_argument('--chain', type=int, default=2200)
    ap.add_argument('--bind', type=int, default=1600)
    ap.add_argument('--str-build', type=int, default=900)
    ap.add_argument('--str-misc', type=int, default=700)
    ap.add_argument('--list-sum', type=int, default=900)
    ap.add_argument('--list-get', type=int, default=500)
    ap.add_argument('--max-mutants', type=int, default=1)
    ap.add_argument('--seed', type=int, default=13)
    ap.add_argument('--out', default='minds/corpus_factory/manifest.jsonl')
    args = ap.parse_args()

    rng = random.Random(args.seed)
    os.makedirs(os.path.dirname(args.out), exist_ok=True)

    counts = {'arith': args.arith, 'loop': args.loop,
              'rec': args.rec, 'chain': args.chain, 'bind': args.bind,
              'str_build': args.str_build, 'str_misc': args.str_misc,
              'list_sum': args.list_sum, 'list_get': args.list_get}
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
