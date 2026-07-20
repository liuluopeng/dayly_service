#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
FLUTTER="flutter"
command -v fvm &>/dev/null && FLUTTER="fvm flutter"
SPLIT="${1:-}"

cd "$ROOT/kongde"
VERSION=$(grep '^version: ' pubspec.yaml | sed 's/version: //')
BUILD_NAME="${VERSION%+*}"
BUILD_NUMBER="${VERSION#*+}"
echo "📦 APK $BUILD_NAME (build $BUILD_NUMBER)"
mkdir -p "$ROOT/dist"

if [ "$SPLIT" = "--split" ]; then
  # 分架构打包，每个 APK 更小
  $FLUTTER build apk --release --split-per-abi \
    --build-name="$BUILD_NAME" --build-number="$BUILD_NUMBER"
  for apk in build/app/outputs/flutter-apk/*.apk; do
    name=$(basename "$apk" | sed "s/app-/kongde-$BUILD_NAME-/")
    cp "$apk" "$ROOT/dist/$name"
    echo "✅ $ROOT/dist/$name"
  done
else
  # 默认只打 arm64-v8a（已在 build.gradle.kts 中配置）
  $FLUTTER build apk --release \
    --build-name="$BUILD_NAME" --build-number="$BUILD_NUMBER"
  cp build/app/outputs/flutter-apk/app-release.apk "$ROOT/dist/kongde-$BUILD_NAME.apk"
  echo "✅ $ROOT/dist/kongde-$BUILD_NAME.apk"
fi
