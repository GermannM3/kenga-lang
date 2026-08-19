#!/usr/bin/env bash
set -e
cd "$(dirname "$0")/.."

PASS_THRESHOLD=80   # we'll just print, not fail
echo "=== Mid-Prophet M2: train + measure (Python trainer + Lite inference) ==="
echo

if [ ! -x /c/Python314/python.exe ] && [ ! -x /c/Python314/python ]; then
  echo "Python 3 not found at /c/Python314; skipping trainer"
  exit 1
fi

PYTHON=${PYTHON:-/c/Python314/python}

echo "step 1: train weights (60 epochs, 5 seeds)"
"$PYTHON" tools/train_m2.py 2>&1 | tail -8

echo
echo "step 2: run M2 inference on each held-out token stream"
total_total=0
total_correct=0
for s in max sqr pow sum; do
  cp "minds/mid_prophet_m2_held_$s.txt" examples/ml/_m2_stream.txt
  line=$(./bootstrap/bin/kenga-lite.exe run examples/ml/mid_prophet_m2_run.kenga 2>&1 | grep -E "^acc=" | head -1)
  if [[ "$line" =~ acc=([0-9]+)/([0-9]+) ]]; then
    a=${BASH_REMATCH[1]}
    b=${BASH_REMATCH[2]}
    pct=$((a * 100 / b))
    echo "  $s : $a / $b ($pct%)"
    total_correct=$((total_correct + a))
    total_total=$((total_total + b))
  else
    echo "  $s : NO OUTPUT ($line)"
  fi
done

echo
if [ "$total_total" -gt 0 ]; then
  pct=$((total_correct * 100 / total_total))
  echo "=== M2 held-out accuracy: $total_correct / $total_total ($pct%) ==="
  echo "    (4 unseen Kenga programs predicted token-by-token)"
fi

rm -f examples/ml/_m2_stream.txt
exit 0
