#!/usr/bin/env bash
# 生成 zici.db（与词典 dict.db 同款：宿主机预生成，生产由 cold_data volume 挂载）
# 包含：生字/生词/词频 + hanzi 笔画 SVG（hanzi-writer-data 数据源）
# 用法: bash scripts/gen-zici-db.sh [输出路径，默认 cold_data/zici.db]
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
mkdir -p cold_data

TARGET="${1:-$ROOT/cold_data/zici.db}"

# 1. 基础表（生字/生词/词频）
cargo run -p lx_dayly_service --bin gen_zici_db "$TARGET"

# 2. hanzi 笔画 SVG 表（node:sqlite 追加）
TMP="$ROOT/target/hanzi-gen"
mkdir -p "$TMP"
if [ ! -d "$TMP/node_modules/hanzi-writer-data" ]; then
  (cd "$TMP" && npm init -y > /dev/null 2>&1 && npm i hanzi-writer-data --no-fund --no-audit 2>&1 | tail -1)
fi
node "$ROOT/scripts/gen-hanzi-db.js" "$TMP/node_modules/hanzi-writer-data" "$TARGET"

echo "zici.db 已生成: $TARGET"
