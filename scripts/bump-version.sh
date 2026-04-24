#!/usr/bin/env bash
# scripts/bump-version.sh — bumpt Version in core, desktop, android synchron.
# Usage: scripts/bump-version.sh 0.1.1

set -euo pipefail

if [ $# -ne 1 ]; then
  echo "Usage: $0 <new-version>"
  echo "Example: $0 0.1.1"
  exit 1
fi

NEW_VERSION="$1"

if ! echo "$NEW_VERSION" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9._-]+)?$'; then
  echo "Error: Version must be semver (e.g. 0.1.1 or 0.1.1-rc1), got '$NEW_VERSION'"
  exit 1
fi

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

echo "Bumping to $NEW_VERSION in:"

# Core Cargo.toml (erste version-Zeile im [package]-Block)
CORE_TOML="$ROOT/core/Cargo.toml"
sed -i -E "0,/^version = \".*\"/ s/^version = \".*\"/version = \"$NEW_VERSION\"/" "$CORE_TOML"
echo "  [core]     $CORE_TOML"

# Desktop Tauri Cargo.toml
DESKTOP_TOML="$ROOT/desktop/src-tauri/Cargo.toml"
sed -i -E "0,/^version = \".*\"/ s/^version = \".*\"/version = \"$NEW_VERSION\"/" "$DESKTOP_TOML"
echo "  [desktop]  $DESKTOP_TOML"

# Desktop Tauri conf
TAURI_CONF="$ROOT/desktop/src-tauri/tauri.conf.json"
if command -v jq >/dev/null 2>&1; then
  tmp="$(mktemp)"
  jq --arg v "$NEW_VERSION" '.version = $v' "$TAURI_CONF" > "$tmp" && mv "$tmp" "$TAURI_CONF"
else
  sed -i -E "s/\"version\"[[:space:]]*:[[:space:]]*\"[^\"]*\"/\"version\": \"$NEW_VERSION\"/" "$TAURI_CONF"
fi
echo "  [tauri]    $TAURI_CONF"

# Desktop package.json (falls existiert)
PKG_JSON="$ROOT/desktop/package.json"
if [ -f "$PKG_JSON" ]; then
  if command -v jq >/dev/null 2>&1; then
    tmp="$(mktemp)"
    jq --arg v "$NEW_VERSION" '.version = $v' "$PKG_JSON" > "$tmp" && mv "$tmp" "$PKG_JSON"
  else
    sed -i -E "s/\"version\"[[:space:]]*:[[:space:]]*\"[^\"]*\"/\"version\": \"$NEW_VERSION\"/" "$PKG_JSON"
  fi
  echo "  [pkg]      $PKG_JSON"
fi

# Android build.gradle.kts
ANDROID_GRADLE="$ROOT/android/app/build.gradle.kts"
sed -i -E "s/versionName = \"[^\"]*\"/versionName = \"$NEW_VERSION\"/" "$ANDROID_GRADLE"
echo "  [android]  $ANDROID_GRADLE"
echo "             (versionCode NICHT geändert — manuell in $ANDROID_GRADLE erhöhen!)"

echo ""
echo "Version $NEW_VERSION gesetzt. Review mit:"
echo "  git diff -- core/Cargo.toml desktop/src-tauri/Cargo.toml desktop/src-tauri/tauri.conf.json desktop/package.json android/app/build.gradle.kts"
