#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
FLUTTER="flutter"
command -v fvm &>/dev/null && FLUTTER="fvm flutter"

cd "$ROOT/kongde"
VERSION=$(grep '^version: ' pubspec.yaml | sed 's/version: //')
BUILD_NAME="${VERSION%+*}"
BUILD_NUMBER="${VERSION#*+}"
echo "📦 APK $BUILD_NAME (build $BUILD_NUMBER)"
mkdir -p "$ROOT/dist"

$FLUTTER build apk --release --split-per-abi \
  --build-name="$BUILD_NAME" --build-number="$BUILD_NUMBER"

for apk in build/app/outputs/flutter-apk/app-*-release.apk; do
  name="kongde-$BUILD_NAME-$(basename "$apk" | sed 's/app-//; s/-release//')"
  cp "$apk" "$ROOT/dist/$name"
  echo "✅ $ROOT/dist/$name"
done
