#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
STATIC="$ROOT/sifu_axuum/static"

echo "=== Building frontends ==="

# 1. Vue
echo ""
echo "--- Vue (webbvueetauri) ---"
cd "$ROOT/webbvueetauri"
pnpm install
pnpm build
rm -rf "$STATIC/vue" && mkdir -p "$STATIC/vue" && cp -r dist/* "$STATIC/vue/"
echo "  -> $STATIC/vue/"

# 2. wasm-demo (Trunk)
echo ""
echo "--- WASM Demo (wasm-demo) ---"
cd "$ROOT/wasm-demo"
which trunk >/dev/null 2>&1 || cargo install trunk
trunk build --release --dist dist
rm -rf "$STATIC/wasm" && mkdir -p "$STATIC/wasm" && cp -r dist/* "$STATIC/wasm/"
echo "  -> $STATIC/wasm/"

# 3. Flutter Web (Rust WASM + Dart2JS)
echo ""
echo "--- Flutter Web (kongde) ---"
cd "$ROOT/kongde"

# 3a. Build Rust WASM module with atomics + shared memory
flutter_rust_bridge_codegen build-web --release \
  --wasm-pack-rustflags \
  "-Ctarget-feature=+atomics,+bulk-memory,+mutable-globals \
   -Clink-arg=--shared-memory \
   -Clink-arg=--import-memory \
   -Clink-arg=--max-memory=33554432 \
   -Clink-arg=--export=__wasm_init_tls \
   -Clink-arg=--export=__tls_size \
   -Clink-arg=--export=__tls_align \
   -Clink-arg=--export=__tls_base \
   -Clink-arg=--export=__heap_base"

# 3b. Patch thread_stack_size default + larger initial memory
sed -i '' \
  -e 's/wasm.__wbindgen_start(thread_stack_size);/wasm.__wbindgen_start(thread_stack_size || 1048576);/' \
  -e 's/initial:[0-9]*,maximum:512/initial:256,maximum:512/' \
  web/pkg/rust_lib_kongde.js

# 3c. Build Flutter web (JS mode, not dart2wasm)
fvm flutter build web --release --base-href=/flutter/
rm -rf "$STATIC/flutter" && mkdir -p "$STATIC/flutter" && cp -r build/web/* "$STATIC/flutter/"
echo "  -> $STATIC/flutter/"

echo ""
echo "=== Done ==="
echo "Run: cd sifu_axuum && cargo run"
echo "Visit: http://localhost:23000/"
