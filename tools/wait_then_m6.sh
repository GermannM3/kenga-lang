#!/usr/bin/env bash
cd "$(dirname "$0")/.." || exit 1
while pgrep -f "realgen_eval.py --model m6" >/dev/null 2>&1; do sleep 60; done
exec bash tools/run_m6_all.sh
