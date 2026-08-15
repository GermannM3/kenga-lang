#!/usr/bin/env bash
# Build Rust-free kenga-lite (Linux / macOS / Git Bash).
set -euo pipefail
cd "$(dirname "$0")"
mkdir -p bin

OUT="bin/kenga-lite"
# On Windows Git Bash, prefer .exe so kenga (MSVC/gnu) finds it
case "$(uname -s 2>/dev/null || echo unknown)" in
  MINGW*|MSYS*|CYGWIN*) OUT="bin/kenga-lite.exe" ;;
esac

CC="${CC:-}"
if [[ -z "$CC" ]]; then
  if command -v cc >/dev/null 2>&1; then CC=cc
  elif command -v clang >/dev/null 2>&1; then CC=clang
  elif command -v gcc >/dev/null 2>&1; then CC=gcc
  else
    echo "No C compiler (cc/clang/gcc). On macOS: xcode-select --install" >&2
    exit 1
  fi
fi

echo "compiling kenga_lite.c → $OUT  ($CC)"
"$CC" -O2 -std=c99 kenga_lite.c -o "$OUT" -lm
echo
"./$OUT"
echo
echo "Try: ./$OUT run examples/selfhost/fact_lite.kenga"
echo "  or: kenga run --lite examples/hello.kenga"
