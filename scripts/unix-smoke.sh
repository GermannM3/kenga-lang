#!/usr/bin/env bash
# Smoke demos for Linux / macOS / Git Bash — run this script, don't paste chat blocks.
set -euo pipefail
cd "$(dirname "$0")/.."

export PATH="${HOME}/.cargo/bin:${PATH}"
# Git Bash on Windows: cargo often lives under /c/Users/<name>/.cargo/bin
if [[ -d /c/Users ]]; then
  for d in /c/Users/*/.cargo/bin; do
    [[ -d "$d" ]] && PATH="$d:$PATH"
  done
  export PATH
fi

if ! command -v kenga >/dev/null 2>&1; then
  echo "kenga not in PATH."
  echo "  Install binary from GitHub Releases, or: cargo install --path . --force"
  echo "  Git Bash: export PATH=\"\$HOME/.cargo/bin:\$PATH\""
  exit 1
fi

echo "==> $(kenga version)"
kenga run examples/ml/autograd_tape.kenga
kenga run examples/ml/mlp_autograd.kenga
kenga run examples/ml/softmax_tape.kenga
kenga run examples/control_elif.kenga

if [[ ! -f bootstrap/bin/kenga-lite && ! -f bootstrap/bin/kenga-lite.exe ]]; then
  echo "==> building lite"
  if command -v cc >/dev/null 2>&1 || command -v gcc >/dev/null 2>&1 || command -v clang >/dev/null 2>&1; then
    bash bootstrap/build.sh
  elif [[ -f bootstrap/build.cmd ]]; then
    cmd.exe //c "bootstrap\\build.cmd"
  else
    echo "no C compiler — skip lite checks"
    echo "==> core demos ok"
    exit 0
  fi
fi

kenga run --lite examples/selfhost/elif_lite.kenga
kenga run --lite examples/hello.kenga
kenga run --lite examples/native_lists.kenga
kenga run --lite examples/selfhost/for_lite.kenga
echo "==> all ok"
