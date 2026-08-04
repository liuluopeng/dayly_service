#!/usr/bin/env bash
# 本地 CI——与 .github/workflows/ci.yml 两个 job 等价
# Rust job: fmt + clippy + test（排除 webbvueetauri；test 另排除需数据库的 lx_dayly_service）
# Vue job: 依赖安装 + wasm-pack + vue-tsc + vite build
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "=== [1/2] Rust: fmt / clippy / test ==="
cargo fmt --all -- --check
cargo clippy --workspace --exclude webbvueetauri -- -D warnings -A future_incompatible
SQLX_OFFLINE=true cargo test --workspace --exclude webbvueetauri --exclude lx_dayly_service --lib --bins

echo "=== [2/2] Vue: wasm / typecheck / build ==="
cd "$ROOT/webbvueetauri"
pnpm install --frozen-lockfile
(cd src/src-wasm && wasm-pack build)
pnpm exec vue-tsc --noEmit
pnpm build

echo ""
echo "本地 CI 全部通过"
