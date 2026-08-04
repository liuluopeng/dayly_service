#!/usr/bin/env bash
# 本地三前端构建：调用共享脚本（scripts/），与 Dockerfile / CI 保持一致
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
STATIC="$ROOT/sifu_axuum/static"

echo "=== Building frontends ==="

# 1. Vue（共享脚本：wasm-pack + pnpm build）
echo ""
echo "--- Vue (webbvueetauri) ---"
bash "$ROOT/scripts/build-vue.sh"
rm -rf "$STATIC/vue" && mkdir -p "$STATIC/vue" && cp -r "$ROOT/webbvueetauri/dist"/* "$STATIC/vue/"
# 与 Docker 一致：同时产出 /dist/
rm -rf "$STATIC/dist" && mkdir -p "$STATIC/dist" && cp -r "$ROOT/webbvueetauri/dist"/* "$STATIC/dist/"
echo "  -> $STATIC/vue/ + $STATIC/dist/"

# 2. wasm-demo (Trunk) —— 仅本地使用，无其他调用方，保持内联
echo ""
echo "--- WASM Demo (wasm-demo) ---"
cd "$ROOT/wasm-demo"
which trunk >/dev/null 2>&1 || cargo install trunk
trunk build --release --dist dist --public-url /wasm/
rm -rf "$STATIC/wasm" && mkdir -p "$STATIC/wasm" && cp -r dist/* "$STATIC/wasm/"
echo "  -> $STATIC/wasm/"

# 3. Flutter Web（共享脚本：FRB build-web + patch + flutter build web）
echo ""
echo "--- Flutter Web (kongde) ---"
cd "$ROOT/kongde"
# FRB codegen（缺失时安装，版本与项目 flutter_rust_bridge 一致）
which flutter_rust_bridge_codegen >/dev/null 2>&1 \
  || cargo install flutter_rust_bridge_codegen --version 2.12.0
# FRB build-web 用 -Z build-std（nightly 特性），确保 rust-src 组件存在
rustup component add rust-src --toolchain nightly 2>/dev/null || true
export FLUTTER="fvm flutter"
bash "$ROOT/scripts/build-flutter-web.sh"
rm -rf "$STATIC/flutter" && mkdir -p "$STATIC/flutter" && cp -r build/web/* "$STATIC/flutter/"
echo "  -> $STATIC/flutter/"

echo ""
echo "=== Done ==="
# 与 sifu_axuum 端口逻辑一致：ENV=development -> 23001，否则 23000；显式 PORT 优先
ENV_MODE="$(grep -E '^ENV=' "$ROOT/sifu_axuum/.env" 2>/dev/null | cut -d= -f2- || echo production)"
PORT_VAL="$(grep -E '^PORT=' "$ROOT/sifu_axuum/.env" 2>/dev/null | cut -d= -f2- || true)"
if [ -n "$PORT_VAL" ]; then
  VISIT_PORT="$PORT_VAL"
elif [ "$ENV_MODE" = "development" ]; then
  VISIT_PORT="23001"
else
  VISIT_PORT="23000"
fi
echo "Run: cd sifu_axuum && cargo run"
echo "Visit: http://localhost:$VISIT_PORT/"
