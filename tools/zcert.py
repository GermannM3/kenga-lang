"""tools/zcert.py — issue/verify Z-certificates for Kenga weight files.

Provenance / convergence note (Hermann directive 24.08):
  Certificate layer is CANONICAL in tools/zmind.py (Z-agent: .km/.kt
  parsers, spectral_marker, tolerance canon F202). This CLI is a thin
  adapter: it delegates issuing/verification to zmind.make_zmind_cert /
  zmind.verify_artifact, which themselves route through the single
  interface zcore.z_verify_unified (exact | tolerant).
  Mid-training issuance is valid: measured L(snap->final)=1.0000
  (ZK-2, minds/corpus_factory/Z_LINEAGE.md).

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

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kenchat
import zmind


def cmd_issue(args):
    _, tensors = kenchat.load_tensors(args.weights)
    cert = zmind.make_zmind_cert(tensors, source_path=args.weights, k=args.k)
    with open(args.out, 'w', encoding='utf-8') as f:
        json.dump(cert, f, indent=1)
    print(f"issued {args.out}: marker={cert['marker']} "
          f"spectral={cert['spectral_marker']} tensors={len(cert['tensors'])}")
    return 0


def cmd_verify(args):
    _, tensors = kenchat.load_tensors(args.weights)
    with open(args.cert, encoding='utf-8') as f:
        cert = json.load(f)
    ok, detail = zmind.verify_artifact(tensors, cert, mode=args.mode)
    print(('VERIFIED' if ok else 'MISMATCH'), f'({args.mode})', detail)
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
