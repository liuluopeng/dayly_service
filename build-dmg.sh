#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
FLUTTER="flutter"
command -v fvm &>/dev/null && FLUTTER="fvm flutter"
cd "$ROOT/kongde"
VERSION=$(grep '^version: ' pubspec.yaml | sed 's/version: //')
BUILD_NAME="${VERSION%+*}"
echo "📦 DMG $BUILD_NAME"
mkdir -p "$ROOT/dist"
$FLUTTER build macos --release --build-name="$BUILD_NAME"
DMG="$ROOT/dist/kongde-$BUILD_NAME.dmg"
if $FLUTTER build macos --help 2>&1 | grep -q '\-\-dmg'; then
  $FLUTTER build macos --release --dmg --build-name="$BUILD_NAME"
  cp build/macos/Build/Products/Release/kongde.dmg "$DMG" 2>/dev/null || true
fi
if [ ! -f "$DMG" ]; then
  APP="build/macos/Build/Products/Release/kongde.app"
  if [ -d "$APP" ]; then
    hdiutil create -volname "Kongde $BUILD_NAME" -srcfolder "$APP" -ov -format UDZO "$DMG"
  fi
fi
echo "✅ $DMG"
