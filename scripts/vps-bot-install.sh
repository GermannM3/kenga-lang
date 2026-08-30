#!/usr/bin/env bash
# Copy service file and print the next commands. Does not write your token.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
echo "repo: $ROOT"
echo
echo "1) bash $ROOT/bootstrap/build.sh"
echo "2) sudo mkdir -p /etc/kenga && sudo tee /etc/kenga/telegram.env <<<'TELEGRAM_BOT_TOKEN=...'"
echo "3) sudo chmod 600 /etc/kenga/telegram.env"
echo "4) edit scripts/kenga-telegram.service (WorkingDirectory / ExecStart ? $ROOT)"
echo "5) sudo cp $ROOT/scripts/kenga-telegram.service /etc/systemd/system/"
echo "6) sudo systemctl enable --now kenga-telegram"
echo
echo "docs: $ROOT/docs/VPS.md"
