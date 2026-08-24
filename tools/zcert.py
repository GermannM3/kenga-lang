"""tools/zcert.py — issue/verify Z-certificates for Kenga weight files.

Provenance: tolerance-certificate semantics follow the Z-system
exp_TELEPORT / F202 ("допуск, не хеш", Hermann directive 24.08); unified
z_verify_unified interface lives in tools/zcore.py (single implementation,
two modes). This CLI is a thin wrapper for .km/.kt-style text weights.

Commands:
  python tools/zcert.py issue  <weights.txt> [--k 32] [--out cert.json]
  python tools/zcert.py verify <weights.txt> --cert cert.json \\
         [--mode exact|tolerant]

Exit code 0 = verified, 1 = mismatch, 2 = error.
"""
import argparse
import json
import os
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kenchat
import zcore

PROVENANCE = ('certificate semantics: tolerance per Z-system F202/'
              'exp_TELEPORT; exact hash per Kenga marker round(W,3); '
              'unified interface zcore.z_verify_unified')


def cmd_issue(args):
    _, tensors = kenchat.load_tensors(args.weights)
    z_full = zcore.z_encode(tensors, 10 ** 9)
    spectra_full = {n: [round(float(x), 8) for x in it['S']]
                    for n, it in z_full['data'].items() if 'S' in it}
    cert = {
        'type': 'zcert-v1',
        'provenance': PROVENANCE,
        'weights_file': args.weights,
        'marker': zcore.z_marker(tensors),
        'k': args.k,
        'created': time.strftime('%Y-%m-%d %H:%M:%S'),
        'spectra': {n: v[:8] for n, v in spectra_full.items()},
        'spectra_full': spectra_full,
    }
    with open(args.out, 'w', encoding='utf-8') as f:
        json.dump(cert, f, indent=1)
    print(f'issued {args.out}: marker={cert["marker"]} tensors={len(spectra_full)}')
    return 0


def cmd_verify(args):
    _, tensors = kenchat.load_tensors(args.weights)
    with open(args.cert, encoding='utf-8') as f:
        cert = json.load(f)
    if args.mode == 'exact':
        # point identity uses RAW weights: SVD roundtrip perturbs values
        # at rounding boundaries and breaks bit-hash by design (F202)
        ok = cert['marker'] == zcore.z_marker(tensors)
    else:
        z_k = zcore.z_encode(tensors, cert.get('k', 32))
        ok = zcore.z_verify_unified(z_k, cert, mode='tolerant')
    print(('VERIFIED' if ok else 'MISMATCH'), f'({args.mode})',
          cert.get('marker', ''))
    return 0 if ok else 1


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest='cmd', required=True)
    p1 = sub.add_parser('issue')
    p1.add_argument('weights')
    p1.add_argument('--k', type=int, default=32)
    p1.add_argument('--out', default=None)
    p2 = sub.add_parser('verify')
    p2.add_argument('weights')
    p2.add_argument('--cert', required=True)
    p2.add_argument('--mode', choices=['exact', 'tolerant'], default='tolerant')
    args = ap.parse_args()
    if args.cmd == 'issue':
        args.out = args.out or args.weights + '.zcert.json'
        return cmd_issue(args)
    return cmd_verify(args)


if __name__ == '__main__':
    sys.exit(main())
