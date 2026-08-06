#!/usr/bin/env bash
# 生成 zici.db（与词典 dict.db 同款：宿主机预生成，生产由 cold_data volume 挂载）
# 用法: bash scripts/gen-zici-db.sh
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
mkdir -p cold_data
cargo run -p lx_dayly_service --bin gen_zici_db cold_data/zici.db
