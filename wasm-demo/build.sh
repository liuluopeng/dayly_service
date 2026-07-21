#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
wasm-pack build --target web --out-dir www/pkg
echo "✅ WASM 构建完成，产物在 www/pkg/"
echo "用任意 HTTP 服务器打开 www/ 目录即可预览"
