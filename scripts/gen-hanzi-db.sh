#!/usr/bin/env bash
# 生成 hanzi 笔画 SVG 数据表（追加进 zici.db）
# 数据源：hanzi-writer-data（npm，每字 JSON 含每笔 SVG path）
# 用法: bash scripts/gen-hanzi-db.sh
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TMP="$ROOT/target/hanzi-gen"
DB_PATH="${1:-$ROOT/cold_data/zici.db}"

mkdir -p "$TMP"
cd "$TMP"
if [ ! -d node_modules/hanzi-writer-data ]; then
  npm init -y > /dev/null 2>&1
  npm i hanzi-writer-data --no-fund --no-audit 2>&1 | tail -1
fi

node "$ROOT/scripts/gen-hanzi-db.js" "$TMP/node_modules/hanzi-writer-data" "$DB_PATH"
echo "hanzi 笔画数据已写入: $DB_PATH"
