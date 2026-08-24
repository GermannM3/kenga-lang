"""tools/zmind.py -- native mind artifacts (.km/.kt) + signed certificates.

PROVENANCE (curator directive): Kenga agent finished Etap 0+1 first
(commit d356141); their tested functions in tools/zcore.py are NOT touched.
The unified verification interface is THEIRS -- zcore.z_verify_unified
(single implementation, two modes); this module uses it and adds the
artifact zone assigned to the Z-agent:

  1. Parsers/writers for Kenga-native mind formats (this agent's zone):
       KENGA_MIND 1   (.km written by the bootstrap C host, rt_prophet.kenga)
       MORE_MIND 1    (.km written by more-VM native_ml.kenga nt_save_mind)
       KENGA_TENSOR 1 (.kt single tensor, rt_tensor.kenga)
       train_m3 .txt  (M-series weights, scale=1000)
     Layout facts taken from the writers; %.17g text parses exactly.
  2. save_mind / load_mind Python counterparts of the language builtins:
     save writes + signs a sidecar (<file>.zmind.json); load verifies and
     REFUSES (ZMindError) on mismatch. Legacy artifacts without a sidecar
     load fine and are reported honestly (passport_verified=False).
  3. Sidecar 'ZMIND 1' carries both reference layers for the ONE interface:
       marker        -> mode='exact'    (point identity, anti-spoofing)
       spectra_full  -> mode='tolerant' (F202: tolerance, not hash;
                                        lossy transfer / lineage)
     plus a compact int16-quantized spectrum copy (spec_q16) whose sha256
     gives a short spectral_marker for logs and quick equality checks.

PRODUCT PROPERTY (curator note, from ZK-2 side result): a certificate may
be issued MID-TRAINING -- it stays valid through to the final checkpoint,
because the spectrum crystallizes early (ZK-2: L(snap, final) = 1.0000
across two independent runs). Sign once during training; verify() holds
until the end.

CLI (thin): python tools/zmind.py sign|verify|show|hash <file> [--mode ...]
Exit codes: 0 ok, 3 mismatch, 4 no certificate, 2 usage/error.
"""
import hashlib
import json
import os
import struct
import time

import numpy as np

import zcore

TOL_S = 5e-5            # default passed through to z_verify_unified
SPEC_INT_MAX = 32767    # int16 max-scaled quantization (compact ref copy)


class ZMindError(Exception):
    """Raised when load_mind refuses an artifact: content mismatch."""


# ================================================== mind IO (.km/.kt/txt) ==

def _read_text(path):
    with open(path, encoding='utf-8') as f:
        return f.read()


def _floats(line):
    return np.array([float(x) for x in line.split()], dtype=np.float64)


def parse_km(text):
    """KENGA_MIND 1 (bootstrap C host, rt_prophet.kenga pl_save_mind)."""
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
    names = ['w1', 'b1', 'w2', 'b2', 'w1_lock', 'b1_lock', 'w2_lock', 'b2_lock']
    shapes = [(h, d), (h,), (d, h), (d,), (h, d), (h,), (d, h), (d,)]
    tensors = {}
    li = 6
    for nm, shp in zip(names, shapes):
        n = int(np.prod(shp))
        arr = _floats(lines[li])
        if len(arr) != n:
            raise ValueError(f'{nm}: expected {n} values, got {len(arr)}')
        tensors[nm] = arr.reshape(shp)
        li += 1
    out = {
        'format': 'KENGA_MIND 1',
        'meta': {'threshold': float(kv['threshold'][0]),
                 'ep_cap': int(kv['ep_cap'][0]),
                 'core_cap': int(kv['core_cap'][0]),
                 'lr': float(kv['lr'][0]),
                 'dim': d, 'hidden': h, 'steps': steps},
        'tensors': tensors,
    }
    tail = lines[li:]          # core/episodic sections kept verbatim
    for jdx, ln in enumerate(tail):
        if ln.startswith('core '):
            out['core_lines'] = tail[jdx:]
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
             f"model {int(m['dim'])} {int(m['hidden'])} "
             f"{int(m.get('steps', 0))}"]

    def emit(arr):
        v = np.asarray(arr, dtype=np.float64).ravel()
        lines.append(' '.join('%.17g' % x for x in v))

    for nm in ('w1', 'b1', 'w2', 'b2',
               'w1_lock', 'b1_lock', 'w2_lock', 'b2_lock'):
        emit(t[nm])
    lines.extend(data.get('core_lines', []))
    with open(path, 'w', encoding='utf-8', newline='\n') as f:
        f.write('\n'.join(lines) + '\n')
    return path


def parse_more_mind(text):
    """MORE_MIND 1 (more-VM, kenga/compiler/native_ml.kenga nt_save_mind)."""
    lines = text.splitlines()
    if not lines or lines[0].strip() != 'MORE_MIND 1':
        raise ValueError('not a MORE_MIND 1 file')
    hdr = lines[1].split()
    thr, cap, hid, lr, dim, steps = (float(hdr[0]), int(hdr[1]), int(hdr[2]),
                                     float(hdr[3]), int(hdr[4]),
                                     int(float(hdr[5])))
    shapes = [(hid, dim), (hid,), (dim, hid), (dim,)]
    names = ['w1', 'b1', 'w2', 'b2']
    tensors = {}
    for idx, (nm, shp) in enumerate(zip(names, shapes)):
        n = int(np.prod(shp))
        arr = _floats(lines[2 + idx])
        if len(arr) != n:
            raise ValueError(f'{nm}: expected {n} values, got {len(arr)}')
        tensors[nm] = arr.reshape(shp)
    ne = int(lines[6])
    eps_lines = lines[7:7 + ne]
    return {'format': 'MORE_MIND 1',
            'meta': {'threshold': thr, 'ep_cap': cap, 'hidden': hid,
                     'lr': lr, 'dim': dim, 'steps': steps,
                     'n_episodes': ne},
            'tensors': tensors, 'episode_lines': eps_lines}


def save_more_mind(path, data):
    """Write MORE_MIND 1 (round-trip safe for parse_more_mind output)."""
    m = data['meta']

    def emit(arr):
        v = np.asarray(arr, dtype=np.float64).ravel()
        return ' '.join('%.17g' % x for x in v)

    lines = ['MORE_MIND 1',
             '%r %d %d %r %d %d' % (m['threshold'], int(m['ep_cap']),
                                    int(m['hidden']), m['lr'],
                                    int(m['dim']), int(m.get('steps', 0)))]
    for nm in ('w1', 'b1', 'w2', 'b2'):
        lines.append(emit(data['tensors'][nm]))
    lines.append(str(len(data.get('episode_lines', []))))
    lines.extend(data.get('episode_lines', []))
    with open(path, 'w', encoding='utf-8', newline='\n') as f:
        f.write('\n'.join(lines) + '\n')
    return path


def parse_kt(text):
    """KENGA_TENSOR 1 -- one tensor per file (rt_tensor.kenga)."""
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
    """train_m3 weights (.txt): float64, divided by scale=1000."""
    info = {}
    tensors = {}
    with open(path, encoding='utf-8') as f:
        first = f.readline().strip()
        for part in first.split():
            key, _, v = part.partition('=')
            try:
                info[key] = int(v)
            except ValueError:
                info[key] = v
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
    """Detect and parse any supported mind artifact."""
    head = _read_text(path).split('\n', 1)[0].strip()
    if head == 'KENGA_MIND 1':
        return parse_km(_read_text(path))
    if head == 'MORE_MIND 1':
        return parse_more_mind(_read_text(path))
    if head == 'KENGA_TENSOR 1':
        return parse_kt(_read_text(path))
    info, tensors = load_txt_weights(path)
    return {'format': 'train_m3 txt', 'meta': info, 'tensors': tensors}


# ------------------------------------- compact spectral reference (q16) ----

def _quantize_spec(s, cut_ratio):
    s = np.asarray(s, dtype=np.float64)
    smax = float(s[0]) if len(s) else 0.0
    if smax <= 0.0:
        return [], 1.0, 0.0
    keep = s > cut_ratio * smax
    kept = s[keep]
    tail = float((s[~keep] ** 2).sum() / max(1e-30, (s ** 2).sum()))
    if len(kept) == 0:
        return [], 1.0, tail
    scale = kept[0] / SPEC_INT_MAX
    q = np.round(kept / scale).astype(np.int64)
    return [int(x) for x in q], float(scale), tail


def spectral_marker(tensors):
    """Short 16-hex id over the int16-quantized spectra (log-friendly)."""
    cut = 1.0 / (SPEC_INT_MAX * 5e-3)
    h = hashlib.sha256()
    for name in sorted(tensors.keys()):
        kind, s = spectrum_of(tensors[name])
        q, scale, _ = _quantize_spec(s, cut)
        qb = struct.pack('<%di' % len(q), *q) if q else b''
        h.update(name.encode('utf-8'))
        h.update(kind.encode())
        h.update(struct.pack('<I', len(q)))
        h.update(qb)
        h.update(struct.pack('<d', scale))
    return h.hexdigest()[:16]


def spectrum_of(arr):
    a = np.asarray(arr, dtype=np.float64)
    if a.ndim >= 2 and min(a.shape) > 1:
        return 'svd', np.linalg.svd(a, compute_uv=False)
    return 'mag', np.sort(np.abs(a.ravel()))[::-1]


# --------------------------- ZMIND 1 certificate over the ONE interface ----

def _sha256_file(path, buf=1 << 20):
    hh = hashlib.sha256()
    with open(path, 'rb') as f:
        while True:
            b = f.read(buf)
            if not b:
                break
            hh.update(b)
    return hh.hexdigest()


def make_zmind_cert(model, source_path=None, k=32):
    """Build the ZMIND 1 certificate (both layers, one verifier).

    Verification always goes through zcore.z_verify_unified:
      mode='exact'    against cert['marker']          (round(W,3) hash)
      mode='tolerant' against cert['spectra_full']    (F202 tolerance canon)

    Adapter notes (why extra fields exist):
      * 'vectors'   -- bias/vector tensors carry no singular values, so the
                       unified comparator never sees them; they are stored
                       exactly here and compared element-wise;
      * 'degenerate'-- all-zero matrices (fresh EWC locks) have zero-norm
                       spectra: cosine is undefined there, so they are
                       checked directly (still zero?) instead of being fed
                       into z_verify_unified.
    """
    sizes = {n: list(np.asarray(a).shape) for n, a in model.items()}
    spectra_full = {}
    spec_q16 = {}
    vectors = {}
    degenerate = []
    for n in sorted(model.keys()):
        kind, s = spectrum_of(model[n])
        spectra_full[n] = [round(float(x), 10) for x in s]
        cut = 1.0 / (SPEC_INT_MAX * 5e-3)
        q, scale, tail = _quantize_spec(s, cut)
        spec_q16[n] = {'kind': kind, 'shape': sizes[n], 'scale': scale,
                       'cut': cut, 'tail_energy': round(tail, 9), 'spec': q}
        if len(s) == 0 or float(s[0]) <= 0.0:
            degenerate.append(n)
        if kind == 'mag':
            vectors[n] = [float(x) for x in np.asarray(model[n]).ravel()]
    return {
        'format': 'ZMIND 1',
        'source_file': source_path,
        'source_format': None,
        'source_sha256': (_sha256_file(source_path)
                          if source_path and os.path.exists(source_path)
                          else None),
        'marker': zcore.z_marker(model),
        'spectral_marker': spectral_marker(model),
        'k': k,
        'created': time.strftime('%Y-%m-%d %H:%M:%S'),
        'tensors': sizes,
        'spectra_full': spectra_full,
        'spec_q16': spec_q16,
        'vectors': vectors,
        'degenerate': degenerate,
    }


def _vectors_match(tensors, cert):
    vecs = cert.get('vectors') or {}
    for name, ref in vecs.items():
        if name not in tensors:
            return False
        act = np.abs(np.asarray(tensors[name], dtype=np.float64)).ravel()
        exp = np.abs(np.asarray(ref, dtype=np.float64))
        if act.shape != exp.shape or not np.allclose(act, exp,
                                                     rtol=0, atol=1e-12):
            return False
    return True


def _degenerate_still_dead(tensors, cert):
    for name in cert.get('degenerate') or []:
        if name not in tensors:
            return False
        if np.any(np.asarray(tensors[name]) != 0):
            return False
    return True


def verify_artifact(tensors, cert, mode='tolerant'):
    """Verify a tensor dict against a ZMIND cert THROUGH the one interface
    zcore.z_verify_unified (+ direct checks for the blind spots its own
    docstring declares: vector tensors and degenerate/zero spectra).
    Returns (ok: bool, detail: str)."""
    z_full = zcore.z_encode(tensors, 10 ** 9)
    if mode == 'exact':
        # F202 canon (kenga-agent patch): bit-hash uses RAW weights.
        # An SVD roundtrip perturbs values at round(3) boundaries and
        # breaks exact identity by design — that is why 'tolerant' exists.
        ok = bool(zcore.z_marker(tensors) == cert.get('marker'))
        return ok, ('exact marker ' +
                    ('match' if ok else f"!= {cert.get('marker')}"))
    if mode != 'tolerant':
        raise ValueError("mode must be 'exact' or 'tolerant'")
    # Feed the unified comparator only HEALTHY matrix tensors: vector
    # tensors carry no S (invisible to it), zero matrices make its cosine
    # undefined (0 even against itself). Both groups are checked directly.
    degen = set(cert.get('degenerate') or [])
    vecs = set((cert.get('vectors') or {}).keys())
    core = {n: tensors[n] for n in tensors
            if n not in degen and n not in vecs}
    ok_s = True
    if core:
        z_core = zcore.z_encode(core, cert.get('k', 32))
        healthy = {'spectra_full': {
            n: v for n, v in (cert.get('spectra_full') or {}).items()
            if n not in degen and n not in vecs}}
        ok_s = bool(zcore.z_verify_unified(z_core, healthy,
                                           mode='tolerant'))
    ok_v = _vectors_match(tensors, cert)
    ok_d = _degenerate_still_dead(tensors, cert)
    sm_ok = spectral_marker(tensors) == cert.get('spectral_marker')
    detail = (f'tolerant={"PASS" if ok_s else "FAIL"} '
              f'vectors={"ok" if ok_v else "CHANGED"} '
              f'degenerate={"dead" if ok_d else "ALIVE?"} '
              f'spectral_marker={"match" if sm_ok else "differs"}')
    return (ok_s and ok_v and ok_d), detail


def cert_path_for(path):
    return str(path) + '.zmind.json'


def sign_file(path, k=32, out_path=None):
    """Parse any supported artifact and write its ZMIND sidecar."""
    data = load_any(path)
    cert = make_zmind_cert(data['tensors'], source_path=path, k=k)
    cert['source_format'] = data['format']
    pp = out_path or cert_path_for(path)
    with open(pp, 'w', encoding='utf-8') as f:
        json.dump(cert, f, indent=1, ensure_ascii=False)
    return cert


def save_mind(path, tensors, meta=None, k=32, sign=True):
    """Python counterpart of the language builtin: write .km/.kt + sign."""
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
        sign_file(path, k=k)
    return path


def load_mind(path, verify=True, expect_marker=None):
    """Load a mind artifact; refuse (raise ZMindError) when its content does
    not match the ZMIND sidecar or an expected spectral marker.

    Artifacts without a sidecar load fine and are reported honestly with
    passport_verified=False (legacy minds stay usable, absence disclosed)."""
    data = load_any(path)
    tensors = data['tensors']
    pp = cert_path_for(path)
    data['passport_verified'] = False
    if verify and os.path.exists(pp):
        with open(pp, encoding='utf-8') as f:
            cert = json.load(f)
        ok, detail = verify_artifact(tensors, cert, mode='tolerant')
        ex_ok, _ = verify_artifact(tensors, cert, mode='exact')
        data['cert_detail'] = detail
        data['exact_marker_match'] = ex_ok
        if not ok:
            raise ZMindError(f'{path}: certificate mismatch: {detail}')
        data['passport_verified'] = True
    elif verify:
        data['passport_note'] = f'no certificate sidecar ({pp})'
    if expect_marker is not None:
        mk = spectral_marker(tensors)
        if mk != expect_marker:
            raise ZMindError(
                f'{path}: identity mismatch: marker {mk} != expected '
                f'{expect_marker}')
        data['passport_verified'] = True
    return data


# ------------------------------------------------------------------- CLI --

def _cli(argv):
    import argparse
    ap = argparse.ArgumentParser(prog='zmind')
    sub = ap.add_subparsers(dest='cmd', required=True)

    s = sub.add_parser('sign', help='compute + write ZMIND sidecar')
    s.add_argument('file')
    s.add_argument('--k', type=int, default=32)

    v = sub.add_parser('verify', help='verify content vs sidecar')
    v.add_argument('file')
    v.add_argument('--mode', choices=['exact', 'tolerant'],
                   default='tolerant')

    h = sub.add_parser('hash', help='print markers')

    for p in (s, v, h):
        p.add_argument('file', nargs='?' if p is h else None)
    args = ap.parse_args(argv)

    if args.cmd == 'sign':
        cert = sign_file(args.file, k=args.k)
        print('signed  :', args.file, '->', cert_path_for(args.file))
        print('marker  :', cert['marker'], '(exact)')
        print('smarker :', cert['spectral_marker'], '(spectrum)')
        return 0
    if args.cmd == 'verify':
        pp = cert_path_for(args.file)
        if not os.path.exists(pp):
            print('no certificate at', pp)
            return 4
        with open(pp, encoding='utf-8') as f:
            cert = json.load(f)
        tensors = load_any(args.file)['tensors']
        ok, detail = verify_artifact(tensors, cert, mode=args.mode)
        print('verdict :', 'PASS' if ok else 'FAIL', f'({args.mode})',
              '|', detail)
        return 0 if ok else 3
    if args.cmd == 'hash':
        t = load_any(args.file)['tensors']
        print('marker  :', zcore.z_marker(t), '(exact)')
        print('smarker :', spectral_marker(t), '(spectrum)')
        return 0
    return 2


if __name__ == '__main__':
    import sys
    sys.exit(_cli(sys.argv[1:]))
