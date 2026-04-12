#!/bin/bash
# NEXUS Core Launcher mit Auto-Restart
# Startet den Daemon und restartet bei Crash (max 5 Versuche in 60s)

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
BINARY="${SCRIPT_DIR}/target/release/nexus-core"
MAX_RESTARTS=5
WINDOW=60

if [ ! -f "$BINARY" ]; then
    echo "Binary nicht gefunden. Baue Release..."
    cd "$SCRIPT_DIR" && cargo build --release || exit 1
fi

restarts=0
window_start=$(date +%s)

while true; do
    echo "[$(date)] NEXUS Core startet..."
    "$BINARY" serve
    exit_code=$?

    if [ $exit_code -eq 0 ]; then
        echo "[$(date)] NEXUS Core normal beendet."
        break
    fi

    now=$(date +%s)
    elapsed=$((now - window_start))

    if [ $elapsed -gt $WINDOW ]; then
        restarts=0
        window_start=$now
    fi

    restarts=$((restarts + 1))

    if [ $restarts -ge $MAX_RESTARTS ]; then
        echo "[$(date)] NEXUS Core $MAX_RESTARTS mal in ${WINDOW}s gecrasht. Stoppe."
        exit 1
    fi

    echo "[$(date)] NEXUS Core gecrasht (exit $exit_code). Neustart $restarts/$MAX_RESTARTS in 2s..."
    sleep 2
done
