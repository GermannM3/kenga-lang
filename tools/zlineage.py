"""tools/zlineage.py — identity drift along the Kenga lineage (ZK-2).

For each pair of checkpoints we compute, per shared-shape tensor:
  * cosine between log(1+S) spectra (truncated SVD, top-k)
  * principal angles between top-k left singular subspaces U
Aggregate lineage score L(pair) = mean_t cos_logS_t * (1 - mean_theta_t/90).

Candidate D8: L >= threshold => "same lineage". Threshold is calibrated
against a random-matrix null (two independent gaussian models of the same
shapes) — see printed report.

Usage:
  python tools/zlineage.py            # runs full family matrix + temporal
"""
import itertools
import json
import os
import sys
import time

import numpy as np

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kenchat

FAMILY = ['m42', 'm5', 'm52', 'm53', 'm6']
TEMPORAL = [('m53', 'minds/mid_prophet_m53_snap_w.txt'),
            ('m6', 'minds/mid_prophet_m6_snap_w.txt')]
TOPK = 32


def load_model(tag_or_path):
    path = tag_or_path if tag_or_path.endswith('.txt') else \
        f'minds/mid_prophet_{tag_or_path}_w.txt'
    if not os.path.exists(path):
        return None
    _, tensors = kenchat.load_tensors(path)
    return tensors


def spectral_signature(tensors, k=TOPK):
    """Per tensor: top-k log(1+S) vector and top-k left subspace U."""
    sig = {}
    for name in sorted(tensors.keys()):
        a = np.asarray(tensors[name], dtype=np.float64)
        if a.ndim < 2 or min(a.shape) < 4 or name == 'E_pos':
            continue  # E_pos differs by K across models; biases are exact
        U, S, _ = np.linalg.svd(a, full_matrices=False)
        kk = min(k, len(S))
        sig[name] = {
            'logs': np.log1p(S[:kk]),
            'U': U[:, :kk],
            'shape': list(a.shape),
        }
    return sig


def pair_drift(sig_a, sig_b):
    """Returns (L, per-tensor detail). Shapes must match; missing skipped."""
    cos_s_list, theta_list = [], []
    detail = {}
    for name in sorted(set(sig_a) & set(sig_b)):
        A, B = sig_a[name], sig_b[name]
        if A['shape'] != B['shape'] or A['logs'].shape != B['logs'].shape:
            continue
        cs = float(A['logs'] @ B['logs'] /
                   ((np.linalg.norm(A['logs']) * np.linalg.norm(B['logs']))
                    + 1e-30))
        M = A['U'].T @ B['U']
        sv = np.linalg.svd(M, compute_uv=False)
        sv = np.clip(sv, 0.0, 1.0)
        thetas = np.degrees(np.arccos(sv))
        mean_theta = float(thetas.mean())
        cos_s_list.append(max(0.0, cs))
        theta_list.append(mean_theta)
        detail[name] = {'cos_logS': round(cs, 4),
                        'mean_theta_deg': round(mean_theta, 2)}
    if not cos_s_list:
        return None, detail
    L = float(np.mean(cos_s_list) * (1.0 - np.mean(theta_list) / 90.0))
    return L, detail


def random_null(shapes_sample, seed=5):
    rng = np.random.RandomState(seed)
    a = {n: rng.randn(*sh) * 0.05 for n, sh in shapes_sample.items()}
    b = {n: rng.randn(*sh) * 0.05 for n, sh in shapes_sample.items()}
    sa, sb = spectral_signature(a), spectral_signature(b)
    L, _ = pair_drift(sa, sb)
    return L


def main():
    models = {}
    shapes_sample = None
    for tag in FAMILY:
        t = load_model(tag)
        if t is None:
            print(f'skip {tag}: weights not found')
            continue
        models[tag] = spectral_signature(t)
        if shapes_sample is None:
            shapes_sample = {n: tuple(s['shape']) for n, s in models[tag].items()}

    pairs = list(itertools.combinations([t for t in FAMILY if t in models], 2))
    # temporal pairs (same run, earlier snapshot vs final)
    for tag, snap in TEMPORAL:
        base = load_model(tag)
        snap_t = load_model(snap)
        if base is None or snap_t is None:
            continue
        key_a, key_b = f'{tag}@snap', f'{tag}@final'
        models[key_a] = spectral_signature(snap_t)
        models[key_b] = models[tag]
        pairs.append((key_a, key_b))

    null_L = random_null(shapes_sample)

    rows = []
    for a, b in pairs:
        if a not in models or b not in models:
            continue
        L, det = pair_drift(models[a], models[b])
        if L is None:
            continue
        rows.append((a, b, L, det))

    rows.sort(key=lambda r: -r[2])
    out = {
        'generated': time.strftime('%Y-%m-%d %H:%M:%S'),
        'null_random_L': round(null_L, 4),
        'pairs': [{'a': a, 'b': b, 'L': round(L, 4), 'detail': det}
                  for a, b, L, det in rows],
    }
    with open('minds/zlineage.json', 'w', encoding='utf-8') as f:
        json.dump(out, f, indent=1)

    print(f'random-null baseline L = {null_L:.4f}')
    print(f'{"pair":24s} {"L(lineage)":>10s}')
    for a, b, L, _ in rows:
        bar = '#' * int(L * 40)
        print(f'{a+" <-> "+b:24s} {L:10.4f} {bar}')

    live = [r for r in rows if '@' not in r[0] and '@' not in r[1]]
    tmp = [r for r in rows if '@' in r[0] or '@' in r[1]]
    if tmp:
        best_tmp = max(tmp, key=lambda r: r[2])
        print(f'\ntemporal (same-run): {best_tmp[0]} <-> {best_tmp[1]} '
              f'L={best_tmp[2]:.4f}')
    if live and tmp:
        lo = min(r[2] for r in live)
        hi = max(r[2] for r in tmp)
        print(f'\nD8 candidate: same-run temporal L ({hi:.3f}) vs '
              f'cross-run family min ({lo:.3f}); null {null_L:.3f}')
    with open('minds/corpus_factory/Z_LINEAGE.md', 'w',
              encoding='utf-8') as f:
        f.write('# Z-Lineage drift (ZK-2)\n\n')
        f.write(f'random null L = {null_L:.4f}\n\n')
        f.write('| pair | L |\n|---|---|\n')
        for a, b, L, _ in rows:
            f.write(f'| {a} <-> {b} | {L:.4f} |\n')
    print('\nwrote minds/zlineage.json + minds/corpus_factory/Z_LINEAGE.md')
    return 0


if __name__ == '__main__':
    sys.exit(main())
