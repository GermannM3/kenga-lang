#!/usr/bin/env bash
# Install Kenga language Telegram bot into /opt/kenga-lang.
# Does not touch /opt/kenga-ai or kenga-bot.service.
set -euo pipefail

if [[ "$(id -u)" -ne 0 ]]; then
  echo "need root" >&2
  exit 1
fi

if systemctl is-active --quiet kenga-bot.service; then
  echo "note: leaving kenga-bot.service (KengaAI) alone"
fi

mkdir -p /opt/kenga-lang
tar -xf /tmp/kenga-lang-bot.tar -C /opt/kenga-lang
rm -f /tmp/kenga-lang-bot.tar

mkdir -p /opt/kenga-lang/minds
mkdir -p /etc/kenga
install -m 640 /tmp/kenga-telegram.env /etc/kenga/telegram.env
rm -f /tmp/kenga-telegram.env
chown root:root /etc/kenga/telegram.env

if ! id -u kengalang >/dev/null 2>&1; then
  useradd --system --home-dir /opt/kenga-lang --no-create-home --shell /usr/sbin/nologin kengalang
fi
chown root:kengalang /etc/kenga/telegram.env
chmod 640 /etc/kenga/telegram.env

chown -R kengalang:kengalang /opt/kenga-lang

echo "building kenga-lite..."
sudo -u kengalang bash /opt/kenga-lang/bootstrap/build.sh

install -m 644 /opt/kenga-lang/scripts/kenga-telegram.service /etc/systemd/system/kenga-lang-bot.service
systemctl daemon-reload

echo "telegram getMe..."
python3 - <<'PY'
import os, json, urllib.request
env = {}
with open("/etc/kenga/telegram.env") as f:
    for line in f:
        line = line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        k, v = line.split("=", 1)
        env[k] = v
tok = env.get("TELEGRAM_BOT_TOKEN", "")
if not tok:
    raise SystemExit("no token in env file")
url = "https://api.telegram.org/bot" + tok + "/getMe"
with urllib.request.urlopen(url, timeout=20) as r:
    data = json.load(r)
print("getMe ok=", data.get("ok"), "username=", (data.get("result") or {}).get("username"))
PY

systemctl enable --now kenga-lang-bot.service
echo "unit: kenga-lang-bot"
systemctl --no-pager --full status kenga-lang-bot.service | head -25
echo "disk:"
du -sh /opt/kenga-lang /opt/kenga-lang/minds
df -h / | tail -1
