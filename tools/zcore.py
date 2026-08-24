"""tools/zcore.py — Z-system core for Kenga models (numpy stack).

Implements the Z x Kenga integration spec v0.1 + v0.2 (Etap 1: passport
before syntax; numpy-first per amendment #2, no C/FFI requirement).

A "model" is a dict name -> np.ndarray (2-D weight matrices and 1-D
biases), exactly what kenchat.load_tensors / train_m3.params_map produce.

Operations (spec §1.1):
    z_encode(model, k)            -> zstate   (per-matrix U,S,V top-k)
    z_decode(zstate, marker=None) -> model
    z_marker(model)               -> str      (16 hex chars)
    z_verify(zstate, marker)      -> bool     (marker of decoded model)
    z_rank(zstate)                -> int      (k used at encode time)
    z_destroy(zstate)             -> zstate   (S = 0)
    z_compose(z1, z2)             -> zstate   (U,V from z1, S from z2)
    z_grow(model, delta_k)        -> model    (re-encode at k+delta)
    z_shrink(model, target_k)     -> model    (re-encode at lower k)
    z_project(model)              -> model    (everything zeroed)
    z_is_alive(model)             -> bool

Marker (spec §3.3): sha256 over float32 little-endian bytes of
round(W, 3) for every entry of every tensor in sorted key order;
first 16 hex chars.
"""
import hashlib
import json
import os
import time

import numpy as np

QUANT = 1000.0  # round(W * QUANT) / QUANT before hashing


def _matrices(model):
    """Sorted (name, array) pairs for deterministic iteration."""
    return [(k, model[k]) for k in sorted(model.keys())]


def z_marker(model):
    h = hashlib.sha256()
    for name, arr in _matrices(model):
        a = np.ascontiguousarray(arr, dtype=np.float32)
        q = np.round(a.astype(np.float64) * QUANT) / QUANT
        h.update(np.ascontiguousarray(q, dtype='<f4').tobytes())
        h.update(name.encode('utf-8'))
    return h.hexdigest()[:16]


def z_encode(model, k):
    """Per-tensor truncated SVD. 1-D tensors (biases) stored exactly."""
    k = int(k)
    z = {'meta': {'k': k, 'created': time.strftime('%Y-%m-%d %H:%M:%S'),
                  'marker': z_marker(model), 'tensors': {}}}
    data = {}
    for name, arr in _matrices(model):
        a = np.asarray(arr, dtype=np.float64)
        if a.ndim < 2:
            data[name] = {'b': a}
            z['meta']['tensors'][name] = {'kind': 'bias', 'shape': list(a.shape)}
            continue
        U, S, Vt = np.linalg.svd(a, full_matrices=False)
        kk = min(k, len(S))
        data[name] = {'U': U[:, :kk].copy(), 'S': S[:kk].copy(),
                      'V': Vt[:kk, :].copy()}
        z['meta']['tensors'][name] = {
            'kind': 'usv', 'shape': list(a.shape), 'k_used': kk,
            's_top8': [round(float(x), 6) for x in S[:8]],
            'energy_top_k': float((S[:kk] ** 2).sum() / max(1e-30, (S ** 2).sum())),
        }
    z['data'] = data
    return z


def z_decode(z, marker=None):
    model = {}
    for name, item in z['data'].items():
        if 'b' in item:
            model[name] = item['b'].astype(np.float64)
            continue
        model[name] = item['U'] @ np.diag(item['S']) @ item['V']
    if marker is not None:
        if not z_verify(z, marker):
            raise ValueError('identity mismatch: marker does not match zstate')
    return model


def z_verify(z, marker):
    try:
        return z_marker(z_decode(z)) == marker
    except Exception:
        return False


def z_rank(z):
    ks = {v.get('k_used') for v in z['meta']['tensors'].values()
          if v.get('kind') == 'usv'}
    return int(max(ks)) if ks else 0


def z_destroy(z):
    zd = {'meta': json.loads(json.dumps(z['meta'])), 'data': {}}
    zd['meta']['destroyed'] = True
    for name, item in z['data'].items():
        if 'S' in item:
            zd['data'][name] = {'U': item['U'], 'V': item['V'],
                                'S': np.zeros_like(item['S'])}
        else:
            zd['data'][name] = {'b': np.zeros_like(item['b'])}
    return zd


def z_compose(z1, z2):
    """U,V from z1; S from z2 (spec §1.1). Same architecture required."""
    zc = {'meta': json.loads(json.dumps(z1['meta'])), 'data': {}}
    zc['meta']['composed_from'] = [z1['meta'].get('marker'),
                                   z2['meta'].get('marker')]
    for name in z1['data']:
        a, b = z1['data'][name], z2['data'][name]
        if 'b' in a:
            zc['data'][name] = {'b': b['b']}
        else:
            assert a['U'].shape == b['U'].shape, f'shape mismatch at {name}'
            zc['data'][name] = {'U': a['U'], 'V': a['V'], 'S': b['S']}
    return zc


def _grow_state(z, delta_k):
    """Expand every USV factor with delta_k fresh orthonormal columns and
    zero singular values (curriculum mechanics, F200): rank capacity grows,
    function is unchanged until training fills the new directions."""
    rng = np.random.RandomState(hash(z['meta'].get('created', '')) % (2**31))
    zd = {'meta': json.loads(json.dumps(z['meta'])), 'data': {}}
    zd['meta']['grown_by'] = int(delta_k)
    for name, item in z['data'].items():
        if 'b' in item:
            zd['data'][name] = {'b': item['b']}
            continue
        U, S, V = item['U'], item['S'], item['V']
        rows, cols = U.shape[0], V.shape[1]
        room_rows = max(0, rows - U.shape[1])
        room_cols = max(0, cols - V.shape[0])
        dk = min(int(delta_k), room_rows, room_cols)
        if dk <= 0:
            zd['data'][name] = {'U': U, 'V': V, 'S': S}
            continue
        Rr = np.linalg.qr(rng.randn(rows, dk))[0]
        Rc = np.linalg.qr(rng.randn(cols, dk))[0]
        zd['data'][name] = {
            'U': np.hstack([U, Rr]),
            'V': np.vstack([V, Rc.T]),
            'S': np.concatenate([S, np.zeros(dk)]),
        }
    return zd


def z_grow(model_or_state, delta_k):
    """Rank-capacity expansion. Accepts a model (re-encoded first) or a
    zstate (expanded in place). Function is preserved; capacity grows."""
    if isinstance(model_or_state, dict) and 'meta' in model_or_state \
            and 'data' in model_or_state:
        return _grow_state(model_or_state, delta_k)
    cur = _current_rank_of_model(model_or_state)
    z = z_encode(model_or_state, cur + int(delta_k))
    return z_decode(_grow_state(z, int(delta_k)))


def _current_rank_of_model(model):
    r = 0
    for arr in model.values():
        a = np.asarray(arr)
        if a.ndim >= 2:
            r = max(r, min(a.shape))
    return r


def z_shrink(model, target_k):
    return z_decode(z_encode(model, target_k))


def z_project(model):
    return {name: np.zeros_like(np.asarray(arr, dtype=np.float64))
            for name, arr in model.items()}


def z_is_alive(model):
    return any(np.any(np.asarray(arr) != 0) for arr in model.values())


# ------------------------------------------------------------- passport ----

def save_zstate(z, path):
    meta = json.dumps(z['meta'], ensure_ascii=False, indent=1)
    arrays = {}
    for name, item in z['data'].items():
        for part, arr in item.items():
            arrays[f'{name}::{part}'] = arr
    np.savez(path, __meta__=np.frombuffer(meta.encode('utf-8'), dtype=np.uint8),
             **arrays)


def load_zstate(path):
    nz = np.load(path, allow_pickle=False)
    meta = json.loads(nz['__meta__'].tobytes().decode('utf-8'))
    data = {}
    for key in nz.files:
        if key == '__meta__':
            continue
        name, part = key.rsplit('::', 1)
        data.setdefault(name, {})[part] = nz[key]
    return {'meta': meta, 'data': data}


def make_passport(model, weights_path, k=32):
    """Lightweight JSON passport for an existing weights file."""
    z = z_encode(model, k)
    sizes = {n: list(np.asarray(a).shape) for n, a in model.items()}
    return {
        'weights_file': weights_path,
        'weights_sha256': _sha256_file(weights_path),
        'marker': z_marker(model),
        'k': k,
        'created': time.strftime('%Y-%m-%d %H:%M:%S'),
        'tensors': sizes,
        'spectra': {n: z['meta']['tensors'][n].get('s_top8')
                    for n in z['meta']['tensors']},
        'energy_top_k': {n: round(z['meta']['tensors'][n].get('energy_top_k', 1.0), 6)
                         for n in z['meta']['tensors']},
    }


def _sha256_file(path, buf=1 << 20):
    import hashlib
    h = hashlib.sha256()
    with open(path, 'rb') as f:
        while True:
            b = f.read(buf)
            if not b:
                break
            h.update(b)
    return h.hexdigest()


def save_passport(passport, path):
    with open(path, 'w', encoding='utf-8') as f:
        json.dump(passport, f, indent=1, ensure_ascii=False)


def load_passport(path):
    with open(path, encoding='utf-8') as f:
        return json.load(f)
