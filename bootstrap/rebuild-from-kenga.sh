#!/usr/bin/env bash
# Chicken-egg: .kenga → emit-c → native binary (needs kenga host once to emit).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

mkdir -p bootstrap/bin
echo "=== emit-c examples/selfhost/kenga_lite.kenga ==="
if command -v kenga >/dev/null 2>&1; then
  kenga emit-c examples/selfhost/kenga_lite.kenga -o bootstrap/kenga_lite.gen.c
elif [[ -x target/release/kenga ]]; then
  target/release/kenga emit-c examples/selfhost/kenga_lite.kenga -o bootstrap/kenga_lite.gen.c
elif [[ -x target/debug/kenga ]]; then
  target/debug/kenga emit-c examples/selfhost/kenga_lite.kenga -o bootstrap/kenga_lite.gen.c
else
  cargo run --quiet -- emit-c examples/selfhost/kenga_lite.kenga -o bootstrap/kenga_lite.gen.c
fi

CC="${CC:-cc}"
command -v "$CC" >/dev/null 2>&1 || CC=gcc
OUT=bootstrap/bin/kenga-lite-gen
case "$(uname -s 2>/dev/null || echo unknown)" in
  MINGW*|MSYS*|CYGWIN*) OUT=bootstrap/bin/kenga-lite-gen.exe ;;
esac

echo "=== compile → $OUT ==="
"$CC" -O2 -std=c99 bootstrap/kenga_lite.gen.c -o "$OUT" -lm
echo
echo "=== run generated lite ==="
"$OUT"
echo
echo "OK: chicken-egg path works (Kenga → C → native)"
