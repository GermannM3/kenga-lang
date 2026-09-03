#!/usr/bin/env bash
set -euo pipefail
chmod 750 /opt/kenga-lang
chmod 700 /opt/kenga-lang/minds
chmod 600 /opt/kenga-lang/minds/tg_*
chown -R kengalang:kengalang /opt/kenga-lang
install -m 644 /opt/kenga-lang/scripts/kenga-telegram.service /etc/systemd/system/kenga-lang-bot.service
sed -i 's/\r$//' /etc/systemd/system/kenga-lang-bot.service
systemctl daemon-reload
python3 - <<'PY'
import json, urllib.request
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
systemctl is-enabled kenga-lang-bot.service
systemctl is-active kenga-lang-bot.service || true
systemctl --no-pager --full status kenga-lang-bot.service | head -20
echo "kenga-ai units untouched:"
systemctl is-active kenga-bot.service kenga-api.service kenga-web.service
