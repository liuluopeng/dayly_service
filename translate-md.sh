#!/bin/bash
# translate-md.sh — 批量翻译英文 Markdown 为中文
# 用法: ./translate-md.sh [-m 模型] [目录]
# 默认: haiku 模型, /Volumes/six/MD 目录

set -euo pipefail

MODEL="haiku"
DIR="/Volumes/six/MD"

while [[ $# -gt 0 ]]; do
    case "$1" in
        -m|--model) MODEL="$2"; shift 2 ;;
        -*) echo "未知选项: $1"; exit 1 ;;
        *) DIR="$1"; shift ;;
    esac
done

OUT_DIR="${DIR}/zh"

mkdir -p "$OUT_DIR"

shopt -s nullglob
files=("$DIR"/*.md)

if [ ${#files[@]} -eq 0 ]; then
    echo "❌ $DIR 下没有 .md 文件"
    exit 1
fi

echo "模型: $MODEL"
echo "找到 ${#files[@]} 个文件"
echo "输出: $OUT_DIR"
echo ""

total=${#files[@]}
ok=0
skipped=0

for ((i=0; i<total; i++)); do
    f="${files[$i]}"
    name=$(basename "$f")
    out="$OUT_DIR/$name"

    # 跳过已翻译的
    if [ -f "$out" ] && [ -s "$out" ]; then
        echo "[$((i+1))/$total] ⏭️  $name (已存在)"
        skipped=$((skipped+1))
        continue
    fi

    # 用 python3 检测中文字符占比（兼容 macOS）
    chinese_pct=$(python3 -c "
import sys
text = open('$f', 'r', errors='replace').read()
total = len(text)
if total < 20:
    print('0')
    sys.exit(0)
chinese = sum(1 for c in text if '一' <= c <= '鿿' or '　' <= c <= '〿' or '＀' <= c <= '￯')
print(str(int(chinese / total * 100)))
" 2>/dev/null)

    if [ -z "$chinese_pct" ]; then
        chinese_pct=0
    fi

    if [ "$chinese_pct" -ge 5 ]; then
        echo "[$((i+1))/$total] ⏭️  $name (中文 ${chinese_pct}%)"
        skipped=$((skipped+1))
        continue
    fi

    echo "[$((i+1))/$total] 🔄 $name (中文 ${chinese_pct}%) ..."

    # 用 claude 翻译
    if cat "$f" | claude --model "$MODEL" -p 'Translate this English markdown to Chinese. Respond with ONLY the translated markdown, no explanations, no greetings, no notes. Keep all image references ![alt](path), links [text](url), code blocks, and markdown formatting exactly as-is. Preserve the attachment/ paths unchanged.' > "$out.tmp" 2>/dev/null; then
        # 去掉可能的 markdown 代码块包装
        sed -i '' -e '/^```/d' "$out.tmp" 2>/dev/null || true
        mv "$out.tmp" "$out"
        echo "      ✅ $name"
        ok=$((ok+1))
    else
        echo "      ❌ $name 失败"
        rm -f "$out.tmp"
    fi

    sleep 2
done

echo ""
echo "完成：${ok} 翻译，${skipped} 跳过，共 ${total}"
echo "输出: $OUT_DIR"
