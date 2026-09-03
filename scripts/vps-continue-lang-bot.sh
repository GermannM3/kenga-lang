#!/usr/bin/env bash
set -euo pipefail
find /opt/kenga-lang -type f \( -name '*.sh' -o -name '*.c' -o -name '*.kenga' -o -name '*.service' -o -name '*.inc.c' \) -print0 | xargs -0 sed -i 's/\r$//'
chown -R kengalang:kengalang /opt/kenga-lang
echo "building kenga-lite..."
sudo -u kengalang bash /opt/kenga-lang/bootstrap/build.sh
ls -l /opt/kenga-lang/bootstrap/bin/kenga-lite
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
systemctl --no-pager --full status kenga-lang-bot.service | head -25
echo "disk:"
du -sh /opt/kenga-lang /opt/kenga-lang/minds
df -h / | tail -1
echo "other kenga units still up:"
systemctl is-active kenga-bot.service kenga-api.service kenga-web.service trade-live.service nginx.service
