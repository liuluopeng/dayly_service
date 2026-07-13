#!/bin/bash
# translate-md.sh — 批量翻译英文 Markdown 为中文
# 用法: ./translate-md.sh [目录]
# 默认目录: /Volumes/six/MD

set -e

DIR="${1:-/Volumes/six/MD}"
OUT_DIR="${DIR}/zh"

mkdir -p "$OUT_DIR"

shopt -s nullglob
files=("$DIR"/*.md)

if [ ${#files[@]} -eq 0 ]; then
    echo "❌ $DIR 下没有 .md 文件"
    exit 1
fi

echo "找到 ${#files[@]} 个文件，开始翻译..."
echo "输出目录: $OUT_DIR"
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

    content=$(cat "$f")
    # 空文件跳过
    if [ -z "$content" ]; then
        echo "[$((i+1))/$total] ⏭️  $name (空文件)"
        skipped=$((skipped+1))
        continue
    fi

    # 检测是否已含中文字符，跳过中文原文
    chinese_count=$(echo "$content" | grep -oP '[\x{4e00}-\x{9fff}]' | wc -l | tr -d ' ')
    total_chars=$(echo -n "$content" | wc -m | tr -d ' ')
    if [ "$total_chars" -gt 0 ] && [ "$((chinese_count * 100 / total_chars))" -ge 5 ]; then
        echo "[$((i+1))/$total] ⏭️  $name (已含中文，跳过)"
        skipped=$((skipped+1))
        continue
    fi

    echo "[$((i+1))/$total] 🔄 $name ..."

    # 用 claude 翻译
    if echo "$content" | claude -p 'Translate this English markdown to Chinese. Respond with ONLY the translated markdown, no explanations, no greetings, no notes. Keep all image references ![alt](path), links [text](url), code blocks, and markdown formatting exactly as-is. Preserve the attachment/ paths unchanged.' > "$out.tmp" 2>/dev/null; then
        # 去掉可能的 markdown 代码块包装
        sed -i '' -e '/^```/d' -e 's/^```markdown//' "$out.tmp" 2>/dev/null || true
        mv "$out.tmp" "$out"
        echo "      ✅ $name"
        ok=$((ok+1))
    else
        echo "      ❌ $name 失败"
        rm -f "$out.tmp"
    fi

    # 间隔，避免 claude 速率限制
    sleep 2
done

echo ""
echo "完成：${ok} 翻译，${skipped} 跳过，共 ${total}"
echo "输出: $OUT_DIR"
