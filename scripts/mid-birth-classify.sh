#!/usr/bin/env bash
set -e
cd "$(dirname "$0")/.."

if [ ! -x bootstrap/bin/kenga-lite.exe ]; then
  echo "build lite first: bootstrap/build.cmd" >&2
  exit 1
fi

PASS=0
TOTAL=9
ARGS=examples/ml/_mid_args.txt

run_test() {
  local probe="$1"
  local expected_idx="$2"
  printf 'examples/ml/kenga_seed_%s.kenga\n' "$probe" > "$ARGS"
  for s in add sub mul fact fib max sqr pow sum; do
    sig=""
    for c in a b c d e f g h i j k l m n o p q r s t u v w x y z; do
      n=$(grep -o "$c" "examples/ml/kenga_seed_$s.kenga" | wc -l)
      sig="$sig:$n"
    done
    echo "${sig#:}" >> "$ARGS"
  done

  out=$(./bootstrap/bin/kenga-lite.exe run examples/ml/mid_prophet_classify.kenga 2>&1 | grep -E "^id=" | head -1)
  if [[ "$out" =~ id=([0-9]+) ]]; then
    id=${BASH_REMATCH[1]}
    if [ "$id" = "$expected_idx" ]; then
      PASS=$((PASS + 1))
      echo "  ok   $probe -> id=$id (expected)"
    else
      echo "  FAIL $probe -> id=$id (expected $expected_idx)"
    fi
  else
    echo "  FAIL $probe -> no id in output: $out"
  fi
}

run_test add  0
run_test sub  1
run_test mul  2
run_test fact 3
run_test fib  4
run_test max  5
run_test sqr  6
run_test pow  7
run_test sum  8

echo
echo "=== mid-prophet classify pass-rate: $PASS / $TOTAL ==="
echo "9 seeds, signature nearest-neighbour (cosine_approx * 10000)."

rm -f "$ARGS"
if [ "$PASS" -ne "$TOTAL" ]; then
  exit 1
fi
exit 0
