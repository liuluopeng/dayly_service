#!/usr/bin/env bash
# 构建 Vue + Wasm 前端——唯一事实来源
# 调用方：build-frontends.sh（本地）、Dockerfile（容器内）、ci.yml / release.yml（GitHub Actions）
# 用法: bash scripts/build-vue.sh
# 产物: webbvueetauri/dist（含 src-wasm wasm-pack 产物）
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT/webbvueetauri"

# 1. 先构建 src-wasm：vite 产物与 vue-tsc 类型检查都依赖 wasm-pack 产物
echo "--- [build-vue] wasm-pack build (src-wasm) ---"
(cd src/src-wasm && wasm-pack build)

# 2. 依赖（CI/Docker 全新环境必装；本地已装则秒过）
echo "--- [build-vue] pnpm install ---"
pnpm install --frozen-lockfile

# 3. 构建（package.json 的 prebuild 含 vue-tsc 类型检查）
echo "--- [build-vue] pnpm build (vue-tsc + vite) ---"
pnpm build

echo "--- [build-vue] 完成: $ROOT/webbvueetauri/dist ---"
