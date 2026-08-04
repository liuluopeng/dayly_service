#!/usr/bin/env bash
# 构建 Flutter Web（FRB wasm + sed patch + flutter build web）——唯一事实来源
# 调用方：build-frontends.sh（本地）、Dockerfile（容器内）
# 用法: bash scripts/build-flutter-web.sh
# 环境: FLUTTER 可覆盖 flutter 命令（本地 fvm 用：FLUTTER="fvm flutter"）
# 产物: kongde/build/web（含 pkg/rust_lib_kongde.js 已 patch）
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT/kongde"
FLUTTER_CMD="${FLUTTER:-flutter}"

# 1. FRB build-web（atomics/shared-memory flags，128MB 内存上限）
echo "--- [build-flutter-web] flutter_rust_bridge_codegen build-web ---"
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

# 2. Patch：thread_stack_size 默认值 + 增大初始化内存（macOS/Linux sed 语法差异）
echo "--- [build-flutter-web] sed patch ---"
if [[ "$(uname)" == "Darwin" ]]; then
  SED=(sed -i '')
else
  SED=(sed -i)
fi
"${SED[@]}" \
  -e 's/wasm.__wbindgen_start(thread_stack_size);/wasm.__wbindgen_start(thread_stack_size || 1048576);/' \
  -e 's/initial:[0-9]*,maximum:[0-9]*/initial:256,maximum:2048/' \
  web/pkg/rust_lib_kongde.js

# 3. Flutter Web（JS 模式，base-href=/flutter/ 部署子路径）
echo "--- [build-flutter-web] flutter build web ---"
$FLUTTER_CMD build web --release --base-href=/flutter/

echo "--- [build-flutter-web] 完成: $ROOT/kongde/build/web ---"
