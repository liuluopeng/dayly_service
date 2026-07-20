#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT/webbvueetauri"

VERSION=$(grep '"version"' src-tauri/tauri.conf.json | head -1 | sed 's/.*: "\(.*\)",/\1/')
echo "📦 Tauri DMG v$VERSION"
mkdir -p "$ROOT/dist"

CI=true pnpm tauri build --bundles dmg 2>&1 | tail -3

TAURI_DMG=$(ls src-tauri/target/release/bundle/dmg/*.dmg 2>/dev/null | head -1)
if [ -n "$TAURI_DMG" ]; then
  cp "$TAURI_DMG" "$ROOT/dist/webbvueetauri-$VERSION.dmg"
  echo "✅ $ROOT/dist/webbvueetauri-$VERSION.dmg"
else
  echo "⚠️  Tauri DMG 未生成"
fi
