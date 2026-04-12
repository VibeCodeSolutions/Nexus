#!/usr/bin/env bash
set -euo pipefail

SERVICE_DIR="$HOME/.config/systemd/user"
SERVICE_FILE="nexus-core.service"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

mkdir -p "$SERVICE_DIR"
cp "$SCRIPT_DIR/$SERVICE_FILE" "$SERVICE_DIR/$SERVICE_FILE"

systemctl --user daemon-reload
systemctl --user enable nexus-core

echo ""
echo "nexus-core service installed and enabled."
echo ""
echo "Start it with:    systemctl --user start nexus-core"
echo "Check status:     systemctl --user status nexus-core"
echo "View logs:        journalctl --user -u nexus-core -f"
