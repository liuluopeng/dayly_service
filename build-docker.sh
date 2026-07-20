#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"

echo "🔨 1/3 编译 WASM（宿主机，不占 Docker 内存）"
cd "$ROOT/webbvueetauri/src/src-wasm"
wasm-pack build
cd "$ROOT"

echo "🔨 2/3 构建 Docker 镜像"
docker compose build

echo "🔨 3/3 启动容器"
docker compose up -d

echo "✅ 完成"
