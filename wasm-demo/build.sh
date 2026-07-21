#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

if [ "${1:-}" = "dev" ]; then
  trunk serve --open
else
  trunk build --release --dist dist
  echo "✅ 构建完成，产物在 dist/"
fi
