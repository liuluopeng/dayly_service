#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT/kongde"

# ── 读取版本号 ──────────────────────────────────────────────
VERSION=$(grep '^version: ' pubspec.yaml | head -1 | sed 's/version: //')
BUILD_NAME="${VERSION%+*}"
BUILD_NUMBER="${VERSION#*+}"
echo "📦 版本: $BUILD_NAME (build $BUILD_NUMBER)"
echo ""

# ── APK ─────────────────────────────────────────────────────
echo "═══════════════════════════════════════════════════════"
echo "  Android APK"
echo "═══════════════════════════════════════════════════════"
fvm flutter build apk --release \
  --build-name="$BUILD_NAME" \
  --build-number="$BUILD_NUMBER"

APK_SRC="build/app/outputs/flutter-apk/app-release.apk"
APK_DST="$ROOT/dist/kongde-$BUILD_NAME.apk"
mkdir -p "$ROOT/dist"
cp "$APK_SRC" "$APK_DST"
echo "✅ APK: $APK_DST"
echo ""

# ── DMG ─────────────────────────────────────────────────────
echo "═══════════════════════════════════════════════════════"
echo "  macOS DMG"
echo "═══════════════════════════════════════════════════════"
fvm flutter build macos --release \
  --build-name="$BUILD_NAME" \
  --build-number="$BUILD_NUMBER"

DMG_SRC="build/macos/Build/Products/Release"
DMG_DST="$ROOT/dist/kongde-$BUILD_NAME.dmg"
mkdir -p "$ROOT/dist"

# 如果 flutter build macos 直接支持 --dmg
if fvm flutter build macos --help 2>&1 | grep -q '\-\-dmg'; then
  fvm flutter build macos --release --dmg \
    --build-name="$BUILD_NAME" \
    --build-number="$BUILD_NUMBER"
  cp "build/macos/Build/Products/Release/kongde.dmg" "$DMG_DST" 2>/dev/null || true
fi

# fallback: 手动打包 .app → .dmg（如果 flutter 不支持 --dmg）
if [ ! -f "$DMG_DST" ]; then
  APP="$DMG_SRC/kongde.app"
  if [ -d "$APP" ]; then
    echo "📦 手动打包 DMG..."
    hdiutil create -volname "Kongde $BUILD_NAME" \
      -srcfolder "$APP" \
      -ov -format UDZO \
      "$DMG_DST"
  fi
fi

if [ -f "$DMG_DST" ]; then
  echo "✅ DMG: $DMG_DST"
else
  echo "⚠️  DMG 未生成，请检查 macOS 构建产物"
fi

echo ""
echo "🎉 完成"
