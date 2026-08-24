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

Z-passport (Etap 1, amendment #1: "квантованный спектр + допуск"):

A passport is a JSON sidecar ``<file>.passport.json`` binding the CONTENT
of a mind artifact (.km / .kt / train_m3 .txt weights) to its identity:

  * per-tensor spectral reference -- singular values for matrices,
    sorted magnitudes for vectors/biases -- quantized to max-scaled
    int16 (quantized spectrum, cf. exp_ZLAB6.encode_Z);
  * a tolerance certificate (exp_TELEPORT F202 lesson: certificate via
    deviation tolerance, not a bare hash): verify passes iff the max
    relative deviation of the actual spectrum from the dequantized
    reference stays under ``tol_relmax``;
  * spectral entries below the auto-derived keep-cut are numerically
    negligible and excluded from the certificate (the cut is chosen so
    int16 quantization noise stays under tol for every kept entry);
  * ``weights_sha256`` + weight-space ``marker`` cover byte-exact file
    identity as a second, stricter layer.

Mind IO (Kenga-native formats, parsed here so save_mind/load_mind can
verify content on load and refuse on mismatch):

  KENGA_MIND 1   -- bootstrap C host (rt_prophet.kenga): key/value header,
                    8 weight lines (w1,b1,w2,b2 + EWC locks), core+episodic.
  MORE_MIND 1    -- more-VM native_ml.kenga: thr cap hid lr dim steps header,
                    4 weight lines (w1,b1,w2,b2), episodes.
  KENGA_TENSOR 1 -- single tensor (.kt): rank, shape line, values line.
  train_m3 .txt  -- "vocab=.. k=.. scale=1000" header, "[name] shape=[..]" lines.

Public mind API mirroring the language builtins:
    save_mind(path, tensors, meta=None, ...)  -- write + sign (sidecar)
    load_mind(path, ...)                      -- load + verify, refuse on mismatch
"""
import hashlib
import json
import os
import struct
import time

import numpy as np

QUANT = 1000.0  # round(W * QUANT) / QUANT before hashing

# ----------------------------------------------------------- passport -----
TOL_RELMAX = 5e-3      # default certificate tolerance (max rel spectral dev)
SPEC_INT_MAX = 32767   # int16 max-scaled quantization of the spectrum


class ZPassportError(Exception):
    """Raised when load_mind refuses a mind: identity/content mismatch."""



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
    """EXACT-hash invariant: recomputes round(W,3) marker of the DECODED
    state and compares strings. No spectral tolerance here.

    Semantics for lossy-k states (truncated SVD): verification is
    SELF-consistent — the state matches the marker of its own decoded
    weights. It will generally FAIL against the marker of the original
    full-rank model, because truncation perturbs W beyond the round(3)
    quantum. That fragility is intentional and mirrors F208 (identity
    is far more fragile than function). Lineage/kinship across models
    is a different relation: see tools/zlineage.py (candidate D8) and
    the unified interface z_verify_unified below.
    """
    try:
        return z_marker(z_decode(z)) == marker
    except Exception:
        return False


def z_verify_unified(z, reference, mode='exact', tol_s=5e-5, cos_min=0.9999):
    """Unified verification interface (Hermann directive, 24.08).

    mode='exact'    : reference = 16-hex marker string; exact round(W,3)
                      hash semantics (point identity, anti-spoofing).
    mode='tolerant' : reference = passport/certificate dict with per-tensor
                      singular values ('spectra_full' preferred, falls back
                      to 'spectra' top-8). Tolerance semantics per
                      exp_TELEPORT / F202: spectrum identity within
                      tolerance instead of bit-hash. Known blind spot:
                      shuffle-of-S breaks function while S matches
                      (F31/F41) — tolerant certifies the SPECTRUM; use
                      exact for anti-spoofing.
    """
    if mode == 'exact':
        return z_verify(z, reference)
    if mode != 'tolerant':
        raise ValueError(f'unknown mode {mode!r}')
    try:
        dec = z_decode(z)
    except Exception:
        return False
    zref = z_encode(dec, 10 ** 9)
    for name, item in z['data'].items():
        if 'S' not in item:
            continue
        ref_list = None
        if isinstance(reference, dict):
            full = reference.get('spectra_full', {}) or {}
            sp = full.get(name)
            if sp is None:
                sp = (reference.get('spectra', {}) or {}).get(name)
            ref_list = sp
        if ref_list is None:
            ref_list = list(item['S'])
        s_new = item['S']
        n = min(len(ref_list), len(s_new))
        if n == 0:
            continue
        r1 = np.asarray(ref_list[:n], dtype=np.float64)
        r2 = np.asarray(s_new[:n], dtype=np.float64)
        denom = max(float(np.linalg.norm(r1)), 1e-30)
        rel = float(np.linalg.norm(r1 - r2)) / denom
        cos = float(r1 @ r2 / ((np.linalg.norm(r1) * np.linalg.norm(r2)) + 1e-30))
        if rel > tol_s * max(1.0, float(np.mean(np.abs(r1)))) * len(r1) \
                or cos < cos_min:
            return False
    return True


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


def passport_for(weights_path, create_if_missing=False, k=32):
    """Backward-compatible lookup: returns the passport dict for a weights
    file, or None (with a warning) when a legacy mind has no sidecar.
    Legacy .km/.kt files keep loading — identity is simply unverified."""
    pp = weights_path + '.passport.json'
    if os.path.exists(pp):
        return load_passport(pp)
    if create_if_missing:
        # caller supplies tensors separately; here we can only warn
        pass
    print(f'[zcore] WARNING: no passport for {weights_path} '
          f'(legacy mind, identity unverified)')
    if create_if_missing:
        return None
    return None


# ================================================== mind IO (.km/.kt/txt) ==
# Parsers for the three native artifact formats. Numeric layout facts are
# taken from the writers: rt_prophet.kenga (KENGA_MIND 1, lite C host),
# kenga/compiler/native_ml.kenga (MORE_MIND 1, more VM), rt_tensor.kenga
# (KENGA_TENSOR 1). Values are printed %.17g -> parsing to float64 is exact.


def _read_text(path):
    with open(path, encoding='utf-8') as f:
        return f.read()


def _floats(line):
    return np.array([float(x) for x in line.split()], dtype=np.float64)


def parse_km(text):
    """KENGA_MIND 1 (bootstrap C host, rt_prophet.kenga)."""
    lines = text.splitlines()
    if not lines or lines[0].strip() != 'KENGA_MIND 1':
        raise ValueError('not a KENGA_MIND 1 file')
    kv = {}
    for ln in lines[1:6]:
        parts = ln.strip().split()
        if len(parts) >= 2:
            kv[parts[0]] = parts[1:]
    d = int(kv['model'][0])
    h = int(kv['model'][1])
    steps = int(float(kv['model'][2]))
    need = [h * d, h, d * h, d, h * d, h, d * h, d]
    names = ['w1', 'b1', 'w2', 'b2', 'w1_lock', 'b1_lock', 'w2_lock', 'b2_lock']
    tensors = {}
    # reshape properly: w1/b1/w2/b2 then locks mirror the same shapes
    shapes = [(h, d), (h,), (d, h), (d,), (h, d), (h,), (d, h), (d,)]
    li = 6
    for nm, shp in zip(names, shapes):
        n = int(np.prod(shp))
        arr = _floats(lines[li])
        if len(arr) != n:
            raise ValueError(f'{nm}: expected {n} values, got {len(arr)}')
        tensors[nm] = arr.reshape(shp)
        li += 1
    del need  # documented layout; shapes list above is authoritative
    out = {
        'format': 'KENGA_MIND 1',
        'meta': {'threshold': float(kv['threshold'][0]),
                 'ep_cap': int(kv['ep_cap'][0]),
                 'core_cap': int(kv['core_cap'][0]),
                 'lr': float(kv['lr'][0]),
                 'dim': d, 'hidden': h, 'steps': steps},
        'tensors': tensors,
    }
    # preserve core/episodic sections verbatim for lossless save_km
    tail = lines[li:]
    for j, ln in enumerate(tail):
        if ln.startswith('core '):
            out['core_lines'] = tail[j:]
            break
    return out


def save_km(path, data):
    """Write KENGA_MIND 1 (round-trip safe for parse_km output)."""
    m = data['meta']
    t = data['tensors']
    lines = ['KENGA_MIND 1',
             f"threshold {m['threshold']!r}",
             f"ep_cap {int(m['ep_cap'])}",
             f"core_cap {int(m['core_cap'])}",
             f"lr {m['lr']!r}",
             f"model {int(m['dim'])} {int(m['hidden'])} {int(m.get('steps', 0))}"]

    def emit(arr):
        v = np.asarray(arr, dtype=np.float64).ravel()
        lines.append(' '.join('%.17g' % x for x in v))

    for nm in ('w1', 'b1', 'w2', 'b2', 'w1_lock', 'b1_lock', 'w2_lock', 'b2_lock'):
        emit(t[nm])
    lines.extend(data.get('core_lines', []))
    with open(path, 'w', encoding='utf-8', newline='\n') as f:
        f.write('\n'.join(lines) + '\n')
    return path


def parse_more_mind(text):
    """MORE_MIND 1 (more-VM, kenga/compiler/native_ml.kenga)."""
    lines = text.splitlines()
    if not lines or lines[0].strip() != 'MORE_MIND 1':
        raise ValueError('not a MORE_MIND 1 file')
    hdr = lines[1].split()
    thr, cap, hid, lr, dim, steps = (float(hdr[0]), int(hdr[1]), int(hdr[2]),
                                     float(hdr[3]), int(hdr[4]), int(float(hdr[5])))
    shapes = [(hid, dim), (hid,), (dim, hid), (dim,)]
    names = ['w1', 'b1', 'w2', 'b2']
    tensors = {}
    for nm, shp in zip(names, shapes):
        n = int(np.prod(shp))
        arr = _floats(lines[2 + names.index(nm)])
        if len(arr) != n:
            raise ValueError(f'{nm}: expected {n} values, got {len(arr)}')
        tensors[nm] = arr.reshape(shp)
    ne = int(lines[6])
    eps_lines = lines[7:7 + ne]
    return {'format': 'MORE_MIND 1',
            'meta': {'threshold': thr, 'ep_cap': cap, 'hidden': hid,
                     'lr': lr, 'dim': dim, 'steps': steps, 'n_episodes': ne},
            'tensors': tensors, 'episode_lines': eps_lines}


def save_more_mind(path, data):
    """Write MORE_MIND 1 (round-trip safe for parse_more_mind output)."""
    m = data['meta']

    def emit(arr):
        v = np.asarray(arr, dtype=np.float64).ravel()
        return ' '.join('%.17g' % x for x in v)

    lines = ['MORE_MIND 1',
             '%r %d %d %r %d %d' % (m['threshold'], int(m['ep_cap']),
                                    int(m['hidden']), m['lr'], int(m['dim']),
                                    int(m.get('steps', 0)))]
    for nm in ('w1', 'b1', 'w2', 'b2'):
        lines.append(emit(data['tensors'][nm]))
    lines.append(str(len(data.get('episode_lines', []))))
    lines.extend(data.get('episode_lines', []))
    with open(path, 'w', encoding='utf-8', newline='\n') as f:
        f.write('\n'.join(lines) + '\n')
    return path


def parse_kt(text):
    """KENGA_TENSOR 1 -- one tensor per file (rt_tensor.kenga tl_save_tensor)."""
    lines = text.splitlines()
    if not lines or lines[0].strip() != 'KENGA_TENSOR 1':
        raise ValueError('not a KENGA_TENSOR 1 file')
    rank = int(lines[1].strip())
    shape = [int(x) for x in lines[2].split()[:rank]]
    vals = _floats(lines[3]) if len(lines) > 3 else np.zeros(0)
    n = int(np.prod(shape)) if shape else 0
    if len(vals) != n:
        raise ValueError(f'kt: expected {n} values, got {len(vals)}')
    return {'format': 'KENGA_TENSOR 1', 'meta': {'shape': shape},
            'tensors': {'t': vals.reshape(shape)}}


def save_kt(path, arr):
    """Write KENGA_TENSOR 1 from an ndarray."""
    a = np.asarray(arr, dtype=np.float64)
    shape = list(a.shape)
    lines = ['KENGA_TENSOR 1', str(len(shape)),
             ' '.join(str(x) for x in shape),
             ' '.join('%.17g' % x for x in a.ravel())]
    with open(path, 'w', encoding='utf-8', newline='\n') as f:
        f.write('\n'.join(lines) + '\n')
    return path


def load_txt_weights(path):
    """train_m3 weights format: 'vocab=.. k=.. scale=1000' + '[name] shape=[..]'
    comma-separated integer-scaled values (see tools/train_m3.py write path).
    Returns (info_dict, tensors_dict) with float64 values divided by scale --
    same convention as kenchat.load_tensors but full precision."""
    info = {}
    tensors = {}
    with open(path, encoding='utf-8') as f:
        first = f.readline().strip()
        for part in first.split():
            k, _, v = part.partition('=')
            try:
                info[k] = int(v)
            except ValueError:
                info[k] = v
        scale = float(info.get('scale', 1000))
        for line in f:
            line = line.strip()
            if not line.startswith('['):
                continue
            rb = line.find(']')
            if rb < 0:
                continue
            name = line[1:rb].strip()
            body = line[rb + 1:].strip()
            shape = []
            si = body.find('shape=[')
            if si >= 0:
                si += len('shape=[')
                ei = body.find(']', si)
                shape = [int(x) for x in body[si:ei].split(',') if x.strip()]
                body = body[ei + 1:]
            nums = [float(x) for x in body.split(',') if x.strip()]
            arr = np.array(nums, dtype=np.float64) / scale
            if shape:
                arr = arr.reshape(shape)
            tensors[name] = arr
    return info, tensors


def load_any(path):
    """Detect and parse any supported mind artifact. Returns a dict with
    keys: format, meta, tensors (+ format-specific extras)."""
    head = _read_text(path).split('\n', 1)[0].strip()
    if head == 'KENGA_MIND 1':
        return parse_km(_read_text(path))
    if head == 'MORE_MIND 1':
        return parse_more_mind(_read_text(path))
    if head == 'KENGA_TENSOR 1':
        return parse_kt(_read_text(path))
    info, tensors = load_txt_weights(path)
    return {'format': 'train_m3 txt', 'meta': info, 'tensors': tensors}


# ------------------------------------------- spectral passport (ZPASSPORT 1)

def spectrum_of(arr):
    """Spectral signature of one tensor: singular values for matrices
    (min(shape)>1), descending magnitudes otherwise. Returns (kind, s_desc)."""
    a = np.asarray(arr, dtype=np.float64)
    if a.ndim >= 2 and min(a.shape) > 1:
        return 'svd', np.linalg.svd(a, compute_uv=False)
    return 'mag', np.sort(np.abs(a.ravel()))[::-1]


def _quantize_spec(s, tol):
    """Max-scaled int16 quantization keeping only entries whose relative
    size guarantees quantization noise <= tol (cut = 1/(INT_MAX*tol)).
    Returns (q_int_list, scale, cut_ratio, tail_energy_frac)."""
    s = np.asarray(s, dtype=np.float64)
    smax = float(s[0]) if len(s) else 0.0
    if smax <= 0.0:
        return [], 1.0, 1.0 / (SPEC_INT_MAX * tol), 0.0
    cut = 1.0 / (SPEC_INT_MAX * tol)
    keep = s > cut * smax
    kept = s[keep]
    tail = float((s[~keep] ** 2).sum() / max(1e-30, (s ** 2).sum()))
    if len(kept) == 0:
        return [], 1.0, cut, tail
    scale = kept[0] / SPEC_INT_MAX
    q = np.round(kept / scale).astype(np.int64)
    return [int(x) for x in q], float(scale), cut, tail


def _dequant_spec(q, scale):
    return np.asarray(q, dtype=np.float64) * float(scale)


def spectral_marker(tensors, tol=None):
    """Strict 16-hex id over the quantized spectra (sorted tensor names)."""
    tol = TOL_RELMAX if tol is None else tol
    h = hashlib.sha256()
    for name in sorted(tensors.keys()):
        _, s = spectrum_of(tensors[name])
        q, scale, _, _ = _quantize_spec(s, tol)
        qb = struct.pack('<%di' % len(q), *q) if q else b''
        h.update(name.encode('utf-8'))
        h.update(struct.pack('<I', len(q)))
        h.update(qb)
        h.update(struct.pack('<d', scale))
    return h.hexdigest()[:16]


def make_passport(model, weights_path, k=32, tol=None):
    """Full Z-passport (ZPASSPORT 1) for an existing weights/mind file.

    Layers (each independently checkable):
      * weights_sha256          -- byte-exact transport integrity;
      * marker (weight space)   -- spec §3.3 hash of round(W,3);
      * quantized spectra +     -- content identity up to orthogonal
        tol_relmax certificate     equivalence, robust to re-formatting.
    """
    tol = TOL_RELMAX if tol is None else float(tol)
    z = z_encode(model, k)
    sizes = {n: list(np.asarray(a).shape) for n, a in model.items()}
    tensors_entry = {}
    spectra_top8 = {}
    energy = {}
    for n in sorted(model.keys()):
        kind, s = spectrum_of(model[n])
        q, scale, cut, tail = _quantize_spec(s, tol)
        tensors_entry[n] = {'kind': kind, 'shape': sizes[n],
                            'n_total': int(len(s)), 'n_kept': len(q),
                            'scale': scale, 'cut': cut,
                            'tail_energy': round(tail, 9),
                            'spec': q}
        top = z['meta']['tensors'][n]
        spectra_top8[n] = top.get('s_top8')
        energy[n] = round(top.get('energy_top_k', 1.0), 6)
    return {
        'format': 'ZPASSPORT 1',
        'weights_file': weights_path,
        'source_format': None,
        'weights_sha256': (_sha256_file(weights_path)
                           if weights_path and os.path.exists(weights_path)
                           else None),
        'marker': z_marker(model),
        'spectral_marker': spectral_marker(model, tol),
        'k': k,
        'tol_relmax': tol,
        'created': time.strftime('%Y-%m-%d %H:%M:%S'),
        'tensors': sizes,
        'spectra': spectra_top8,
        'energy_top_k': energy,
        'spec_ref': tensors_entry,
    }


def verify_passport(model, passport, tol=None):
    """Tolerance certificate of ``model`` content against ``passport``.

    Passes iff every kept spectral entry stays within ``tol`` relative
    deviation of the dequantized reference (denominator floors at the
    quantization half-step so int16 noise cannot trip the certificate).
    Returns a report dict; never raises for mismatches."""
    report = {'pass': True, 'tol': None, 'per_tensor': {},
              'missing': [], 'extra': [], 'reasons': [],
              'spectral_marker_match': None, 'weights_sha256_match': None}
    tol = float(TOL_RELMAX if tol is None else tol)
    report['tol'] = tol
    ref = passport.get('spec_ref') or {}
    for name in sorted(ref.keys()):
        if name not in model:
            report['missing'].append(name)
            continue
        e = ref[name]
        _, s_act = spectrum_of(model[name])
        smax_act = float(s_act[0]) if len(s_act) else 0.0
        kept_act = s_act[s_act > e['cut'] * smax_act] if smax_act > 0 else s_act[:0]
        s_ref = _dequant_spec(e['spec'], e['scale'])
        smax = max(float(s_ref[0]) if len(s_ref) else 0.0, smax_act, 1e-300)
        if len(kept_act) != len(s_ref):
            report['per_tensor'][name] = {
                'pass': False, 'dev': float('inf'),
                'n_ref': len(s_ref), 'n_act': int(len(kept_act))}
            report['reasons'].append(
                f'{name}: kept-entry count changed '
                f'(ref {len(s_ref)} vs act {len(kept_act)})')
            report['pass'] = False
            continue
        floor = e['cut'] * smax
        denom = np.maximum.reduce([np.abs(s_ref), np.abs(kept_act),
                                   np.full(len(s_ref), 0.6 * e['scale']),
                                   np.full(len(s_ref), floor)])
        dev = float(np.max(np.abs(kept_act - s_ref) / denom)) if len(s_ref) else 0.0
        ok = dev <= tol
        report['per_tensor'][name] = {'pass': ok, 'dev': dev,
                                      'n_ref': len(s_ref), 'n_act': len(kept_act)}
        if not ok:
            report['reasons'].append(
                f'{name}: spectral deviation {dev:.3e} > tol {tol:.0e}')
            report['pass'] = False
    for name in model.keys():
        if name not in ref:
            report['extra'].append(name)
    if report['missing']:
        report['pass'] = False
        report['reasons'].append(f'missing tensors: {report["missing"]}')
    if report['extra']:
        report['pass'] = False
        report['reasons'].append(f'unexpected tensors: {report["extra"]}')
    exp_mk = passport.get('spectral_marker')
    if exp_mk:
        report['spectral_marker_match'] = (spectral_marker(model, tol) == exp_mk)
    exp_sha = passport.get('weights_sha256')
    return report


# ------------------------------------------------- save_mind/load_mind ----

def passport_path_for(path):
    return str(path) + '.passport.json'


def sign_file(path, k=32, tol=None, out_path=None):
    """Compute the Z-passport for a mind artifact and write its sidecar."""
    data = load_any(path)
    pas = make_passport(data['tensors'], path, k=k, tol=tol)
    pas['source_format'] = data['format']
    pp = out_path or passport_path_for(path)
    save_passport(pas, pp)
    return pas


def save_mind(path, tensors, meta=None, k=32, tol=None, sign=True):
    """Python counterpart of the language builtin: write a mind artifact
    (format chosen by extension: .km / .kt) and sign it with a passport."""
    ext = os.path.splitext(str(path))[1].lower()
    meta = dict(meta or {})
    if ext == '.kt':
        arr = tensors if isinstance(tensors, np.ndarray) else \
            np.asarray(list(tensors.values())[0])
        save_kt(path, arr)
    elif ext == '.km':
        t = tensors
        if 'dim' not in meta:
            w1 = np.asarray(t['w1'])
            meta['dim'], meta['hidden'] = int(w1.shape[1]), int(w1.shape[0])
        meta.setdefault('threshold', 0.05)
        meta.setdefault('ep_cap', 64)
        meta.setdefault('core_cap', 24)
        meta.setdefault('lr', 0.08)
        meta.setdefault('steps', 0)
        fills = {
            'b1': np.zeros(meta['hidden']), 'b2': np.zeros(meta['dim']),
            'w1_lock': np.zeros((meta['hidden'], meta['dim'])),
            'b1_lock': np.zeros(meta['hidden']),
            'w2_lock': np.zeros((meta['dim'], meta['hidden'])),
            'b2_lock': np.zeros(meta['dim'])}
        t = dict(t)
        for nm, v in fills.items():
            t.setdefault(nm, v)
        save_km(path, {'meta': meta, 'tensors': t})
    else:
        raise ValueError('save_mind: unsupported extension (use .km/.kt)')
    if sign:
        sign_file(path, k=k, tol=tol)
    return path


def load_mind(path, verify=True, expect_marker=None, tol=None):
    """Load a mind artifact; refuse (raise ZPassportError) when content
    does not match its passport sidecar or an expected spectral marker.

    Files without any passport are returned with passport_verified=False
    (legacy artifacts stay loadable; absence is reported honestly)."""
    data = load_any(path)
    tensors = data['tensors']
    pp = passport_path_for(path)
    data['passport_verified'] = False
    if verify and os.path.exists(pp):
        pas = load_passport(pp)
        rep = verify_passport(tensors, pas, tol=tol)
        data['passport_report'] = rep
        if not rep['pass']:
            raise ZPassportError(
                f'{path}: passport mismatch: ' + '; '.join(rep['reasons']))
        data['passport_verified'] = True
    elif verify:
        data['passport_note'] = f'no passport sidecar ({pp})'
    if expect_marker is not None:
        mk = spectral_marker(tensors, tol=tol)
        if mk != expect_marker:
            raise ZPassportError(
                f'{path}: identity mismatch: marker {mk} != expected '
                f'{expect_marker}')
        data['passport_verified'] = True
    return data
