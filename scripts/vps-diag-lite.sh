#!/usr/bin/env bash
set -u
cd /opt/kenga-lang
LITE=./bootstrap/bin/kenga-lite
run() {
  local name="$1" file="$2"
  echo "=== $name ==="
  sudo -u kengalang "$LITE" run "$file"
  echo "exit=$?"
}

cat > /tmp/h.kenga <<'EOF'
fn main() -> i64 { println(1); return 0; }
EOF
run hello /tmp/h.kenga

cat > /tmp/sw.kenga <<'EOF'
fn main() -> i64 { println(starts_with("ab", "a")); return 0; }
EOF
run starts_with /tmp/sw.kenga

cat > /tmp/mind.kenga <<'EOF'
fn main() -> i64 {
    let m = memory_config(0.12, 8, 8);
    println(1);
    return 0;
}
EOF
run mind /tmp/mind.kenga

run bot examples/telegram_bot.kenga
