#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
STATIC="$ROOT/sifu_axuum/static"

echo "=== Building frontends ==="

# 1. Vue
echo ""
echo "--- Vue (webbvueetauri) ---"
cd "$ROOT/webbvueetauri"
# 先构建 src-wasm：vite 产物依赖它，且保证 vue-tsc（prebuild）类型检查用的是最新 wasm 绑定
(cd src/src-wasm && wasm-pack build)
pnpm install
pnpm build
rm -rf "$STATIC/vue" && mkdir -p "$STATIC/vue" && cp -r dist/* "$STATIC/vue/"
# 与 Docker 一致：同时产出 /dist/（Docker 内构建也覆盖这两处）
rm -rf "$STATIC/dist" && mkdir -p "$STATIC/dist" && cp -r dist/* "$STATIC/dist/"
echo "  -> $STATIC/vue/ + $STATIC/dist/"

# 2. wasm-demo (Trunk)
echo ""
echo "--- WASM Demo (wasm-demo) ---"
cd "$ROOT/wasm-demo"
which trunk >/dev/null 2>&1 || cargo install trunk
# --public-url /wasm/：产物引用加 /wasm/ 前缀，否则部署在子路径下 JS/CSS 404
trunk build --release --dist dist --public-url /wasm/
rm -rf "$STATIC/wasm" && mkdir -p "$STATIC/wasm" && cp -r dist/* "$STATIC/wasm/"
echo "  -> $STATIC/wasm/"

# 3. Flutter Web (Rust WASM + Dart2JS)
echo ""
echo "--- Flutter Web (kongde) ---"
cd "$ROOT/kongde"

# 检查 FRB codegen（缺失时安装，版本与项目 flutter_rust_bridge 一致）
which flutter_rust_bridge_codegen >/dev/null 2>&1 \
  || cargo install flutter_rust_bridge_codegen --version 2.12.0

# FRB build-web 用 -Z build-std（nightly 特性），确保 rust-src 组件存在
rustup component add rust-src --toolchain nightly 2>/dev/null || true

# 3a. Build Rust WASM module with atomics + shared memory
flutter_rust_bridge_codegen build-web --release \
  --wasm-pack-rustflags \
  "-Ctarget-feature=+atomics,+bulk-memory,+mutable-globals \
   -Clink-arg=--shared-memory \
   -Clink-arg=--import-memory \
   -Clink-arg=--max-memory=134217728 \
   -Clink-arg=--export=__wasm_init_tls \
   -Clink-arg=--export=__tls_size \
   -Clink-arg=--export=__tls_align \
   -Clink-arg=--export=__tls_base \
   -Clink-arg=--export=__heap_base"

# 3b. Patch thread_stack_size default + larger initial memory
sed -i '' \
  -e 's/wasm.__wbindgen_start(thread_stack_size);/wasm.__wbindgen_start(thread_stack_size || 1048576);/' \
  -e 's/initial:[0-9]*,maximum:[0-9]*/initial:256,maximum:2048/' \
  web/pkg/rust_lib_kongde.js

# 3c. Build Flutter web (JS mode, not dart2wasm)
fvm flutter build web --release --base-href=/flutter/
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
