#!/usr/bin/env bash
set -euo pipefail
mkdir -p /opt/kenga-lang/kenga/compiler /opt/kenga-lang/kenga/emit
tar -xf /tmp/kenga-lang-patch.tar -C /opt/kenga-lang
rm -f /tmp/kenga-lang-patch.tar
find /opt/kenga-lang -type f \( -name '*.kenga' -o -name '*.c' -o -name '*.inc.c' -o -name '*.sh' -o -name '*.service' \) -print0 | xargs -0 sed -i 's/\r$//'
# systemd env files need a trailing newline
python3 - <<'PY'
p = "/etc/kenga/telegram.env"
with open(p, "rb") as f:
    b = f.read()
if not b.endswith(b"\n"):
    with open(p, "ab") as f:
        f.write(b"\n")
PY
chown -R kengalang:kengalang /opt/kenga-lang
chmod 750 /opt/kenga-lang
chmod 700 /opt/kenga-lang/minds
chmod 600 /opt/kenga-lang/minds/tg_* 2>/dev/null || true
echo "rebuild kenga-lite"
sudo -u kengalang mkdir -p /opt/kenga-lang/bootstrap/bin
sudo -u kengalang bash -c 'cd /opt/kenga-lang/bootstrap && cc -O2 -std=c99 -D_DEFAULT_SOURCE kenga_lite.c -o bin/kenga-lite -lm'
test -x /opt/kenga-lang/bootstrap/bin/kenga-lite
# more RAM for first compile of native_ml + bot
if ! grep -q MemoryMax /etc/systemd/system/kenga-lang-bot.service; then
  true
fi
sed -i 's/^MemoryMax=.*/MemoryMax=1500M/' /etc/systemd/system/kenga-lang-bot.service
systemctl daemon-reload
systemctl reset-failed kenga-lang-bot.service || true
systemctl start kenga-lang-bot.service
echo "started"
systemctl is-active kenga-lang-bot.service || true
echo "kenga-ai still:"
systemctl is-active kenga-bot.service kenga-api.service kenga-web.service nginx.service
echo "tree:"
du -sh /opt/kenga-lang /opt/kenga-lang/kenga /opt/kenga-lang/minds
ls -l /opt/kenga-lang/kenga/compiler
