#!/bin/bash
# =============================================================
# NEXUS — GitHub Setup Script
# Einmalig ausführen im Nexus-Projektordner auf deinem Rechner.
# Voraussetzung: gh CLI installiert + authentifiziert (gh auth login)
# =============================================================

set -e

echo "🔧 NEXUS GitHub Setup startet..."

# 1. Git initialisieren (falls noch nicht geschehen)
if [ ! -d ".git" ]; then
    git init -b main
    echo "✅ Git-Repo initialisiert (Branch: main)"
else
    echo "ℹ️  Git-Repo existiert bereits"
fi

# 2. Alle Dateien stagen
git add -A

# 3. Initial Commit
git commit -m "feat: Phase 0 — Projekt-Setup & Architektur-Gerüst

- Mono-Repo-Struktur: /core (Rust), /android (Kotlin), /docs
- Rust-Daemon Stub mit axum (GET /health)
- SQLite-Migration Stub (001_braindump.sql)
- NEXUS_Masterplan.md + CURRENT_STATE.md
- README, LICENSE (MIT), .gitignore"

echo "✅ Initial Commit erstellt"

# 4. GitHub-Repo erstellen + Push
gh repo create nexus-os --public --source=. --remote=origin --push

echo ""
echo "🚀 Done! Repo: https://github.com/$(gh api user --jq '.login')/nexus-os"
echo ""
echo "Nächster Schritt: Phase 0 DoD abarbeiten (Rust kompiliert, Android kompiliert)"
