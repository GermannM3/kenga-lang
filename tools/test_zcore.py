"""tools/test_zcore.py — unit tests per Z x Kenga spec §8.1.

Run: python tools/test_zcore.py
No pytest dependency; exits non-zero on failure.
"""
import os
import sys
import time

import numpy as np

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import zcore


def tiny_model(V=32, K=16, D=24, L=2, seed=11):
    rng = np.random.RandomState(seed)
    m = {'E_tok': rng.randn(V, D) * 0.3,
         'E_pos': rng.randn(K, D) * 0.3,
         'Wout': rng.randn(D, V) * 0.3,
         'bout': rng.randn(V) * 0.1}
    for li in range(L):
        m[f'{li}:Wq'] = rng.randn(D, D) * 0.2
        m[f'{li}:Wo'] = rng.randn(D, D) * 0.2
        m[f'{li}:W1'] = rng.randn(D, D * 2) * 0.2
        m[f'{li}:b1'] = rng.randn(D * 2) * 0.05
    return m


def rmse(a, b):
    ka, kb = sorted(a.keys()), sorted(b.keys())
    assert ka == kb, 'model key sets differ'
    va = np.concatenate([np.asarray(a[k]).ravel() for k in ka])
    vb = np.concatenate([np.asarray(b[k]).ravel() for k in kb])
    return float(np.sqrt(((va - vb) ** 2).mean()))


def agreement(pred_a, pred_b, n=50, seed=3):
    """Mean cosine of linear responses on random inputs (hybrid vs donor)."""
    rng = np.random.RandomState(seed)
    keys = sorted(pred_a.keys())
    vals = []
    for _ in range(n):
        x = {k: rng.randn(*pred_a[k].shape) * 0.5 for k in keys}
        va = np.concatenate([(x[k] * pred_a[k]).ravel() for k in keys])
        vb = np.concatenate([(x[k] * pred_b[k]).ravel() for k in keys])
        na, nb = np.linalg.norm(va), np.linalg.norm(vb)
        if na < 1e-30 or nb < 1e-30:
            continue
        vals.append(max(0.0, float(va @ vb / (na * nb))))
    return float(np.mean(vals)) if vals else 0.0


def zg_s_data(z):
    return z['data']


def main():
    results = []

    def check(name, ok, detail=''):
        results.append((name, ok, detail))
        print(f'[{"PASS" if ok else "FAIL"}] {name} {detail}')

    m = tiny_model()

    # 1. encode_decode_roundtrip at full rank: RMSE < 1e-5 (spec D1/D2)
    t0 = time.time()
    z_full = zcore.z_encode(m, 10 ** 9)
    dec = zcore.z_decode(z_full)
    e = rmse(m, dec)
    check('encode_decode_roundtrip', e < 1e-5, f'RMSE={e:.2e} ({time.time()-t0:.2f}s)')

    # 2. marker_stability
    mk1 = zcore.z_marker(m)
    mk2 = zcore.z_marker(dict(reversed(list(m.items()))))
    check('marker_stability', mk1 == mk2, f'marker={mk1}')

    # 3. marker_sensitivity (A1: one flipped weight -> different marker)
    m2 = {k: v.copy() for k, v in m.items()}
    first_key = sorted(m2.keys())[0]
    m2[first_key].ravel()[0] += 0.01
    check('marker_sensitivity', zcore.z_marker(m2) != mk1)

    # 4. truncated rank honored
    z8 = zcore.z_encode(m, 8)
    check('z_rank', zcore.z_rank(z8) <= 8, f'rank={zcore.z_rank(z8)}')

    # 5. verify own marker on truncated state (identity preserved under lossy k)
    own = zcore.z_verify(z8, zcore.z_marker(zcore.z_decode(z8)))
    check('verify_own_truncated', own)

    # 6. foreign marker rejected (A1 via D5-style separation)
    foreign = zcore.z_verify(z8, '0123456789abcdef')
    check('verify_foreign_rejected', not foreign)

    # 7. project_zeros (D3) + is_alive
    dead = zcore.z_project(m)
    check('project_zeros', not zcore.z_is_alive(dead))
    check('is_alive_original', zcore.z_is_alive(m))

    # 8. destroy_kills (D6): S=0 -> decode gives zero model
    zd = zcore.z_destroy(z_full)
    dm = zcore.z_decode(zd)
    check('destroy_kills', not zcore.z_is_alive(dm))

    # 9. compose_functional (D7 proxy): U,V from A carry geometry;
    #    decoded hybrid stays close to A (agreement >= 0.85)
    za = zcore.z_encode(tiny_model(seed=11), 10 ** 9)
    zb = zcore.z_encode(tiny_model(seed=99), 10 ** 9)
    zc = zcore.z_compose(za, zb)
    hyb = zcore.z_decode(zc)
    a_dec = zcore.z_decode(za)
    agr = agreement(hyb, a_dec)
    check('compose_functional', agr >= 0.85, f'agreement={agr:.3f}')
    mk_hyb_ok = zcore.z_verify(zc, zcore.z_marker(hyb))
    mk_foreign = zcore.z_verify(zc, mk1)
    check('compose_identity_new', mk_hyb_ok and not mk_foreign)

    # 10. grow: from truncated state, capacity grows, function unchanged
    small = zcore.z_decode(zcore.z_encode(m, 4))
    e_small = rmse(m, small)
    z4 = zcore.z_encode(m, 4)
    len_before = max(len(z4['data'][n].get('S', [])) for n in z4['data'])
    z_grown = zcore._grow_state({'meta': dict(z4['meta']),
                                 'data': {n: {p: a.copy() for p, a in it.items()}
                                          for n, it in z4['data'].items()}}, 8)
    grown = zcore.z_decode(z_grown)
    e_grown = rmse(m, grown)
    len_after = max(len(z_grown['data'][n].get('S', [])) for n in z_grown['data'])
    check('grow_capacity_and_function',
          abs(e_grown - e_small) < 1e-9 and len_after > len_before,
          f'function preserved (rmse {e_small:.4f}=={e_grown:.4f}), '
          f'S-length {len_before}->{len_after}')

    # 11. save/load zstate round trip
    p = 'minds/_ztest_state.npz'
    os.makedirs('minds', exist_ok=True)
    zcore.save_zstate(z8, p)
    z8b = zcore.load_zstate(p)
    e_rt = rmse(zcore.z_decode(z8), zcore.z_decode(z8b))
    check('save_load_zstate', e_rt == 0.0, f'RMSE={e_rt}')

    # 12. REAL production artifact: M5.3 weights
    w53 = 'minds/mid_prophet_m53_w.txt'
    if os.path.exists(w53):
        sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
        import kenchat
        _, tensors = kenchat.load_tensors(w53)
        t0 = time.time()
        pas = zcore.make_passport(tensors, w53, k=32)
        dt = time.time() - t0
        pp = w53 + '.passport.json'
        zcore.save_passport(pas, pp)
        pas2 = zcore.load_passport(pp)
        ok_marker = pas2['marker'] == zcore.z_marker(
            kenchat.load_tensors(w53)[1])
        check('real_m53_passport', ok_marker and len(pas2['spectra']) > 5,
              f"marker={pas2['marker']} tensors={len(pas2['spectra'])} "
              f'svd_time={dt:.1f}s')

    print()
    fails = [r for r in results if not r[1]]
    print(f'TOTAL: {len(results) - len(fails)}/{len(results)} passed')
    return 1 if fails else 0


if __name__ == '__main__':
    sys.exit(main())
