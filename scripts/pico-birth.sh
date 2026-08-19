#!/usr/bin/env bash
set -e
cd "$(dirname "$0")/.."

if [ ! -x bootstrap/bin/kenga-lite.exe ]; then
  echo "build lite first: bootstrap/build.cmd" >&2
  exit 1
fi

PASS=0
TOTAL=5
ARGS=examples/ml/_pico_args.txt

run_test() {
  local prompt="$1"
  local seed="$2"
  local want="$3"
  local out="examples/ml/pico_born_$(echo "$seed" | tr '/' '_').kenga"

  printf '%s\n%s\n%s\n' "$prompt" "$seed" "$out" > "$ARGS"

  echo "--- prompt \"$prompt\" | seed $seed | want $want ---"

  if ! bootstrap/bin/kenga-lite.exe run examples/ml/pico_birth_single.kenga > /tmp/pico_gen.out 2>&1; then
    echo "  FAIL suffix-LM"
    cat /tmp/pico_gen.out
    return
  fi

  if [ ! -f "$out" ]; then
    echo "  FAIL: $out not created"
    return
  fi

  local got
  got=$(bootstrap/bin/kenga-lite.exe run "$out" 2>&1 | head -1)
  if [ "$got" = "$want" ]; then
    echo "  ok   want $want  got $got"
    PASS=$((PASS + 1))
  else
    echo "  FAIL want $want got $got"
  fi

  rm -f "$out"
}

run_test "fn add"  examples/ml/kenga_seed_add.kenga  5
run_test "fn sub"  examples/ml/kenga_seed_sub.kenga  7
run_test "fn mul"  examples/ml/kenga_seed_mul.kenga  42
run_test "fn fact" examples/ml/kenga_seed_fact.kenga 120
run_test "fn fib"  examples/ml/kenga_seed_fib.kenga  21

echo
echo "=== pico-birth pass-rate: $PASS / $TOTAL ==="

# Cleanup
rm -f "$ARGS"

if [ "$PASS" -ne "$TOTAL" ]; then
  exit 1
fi
echo "all seeds compile and produce expected value"
exit 0
