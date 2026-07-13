---
name: translate-md
description: Translate all English markdown files in a directory to Chinese. Scans the entire directory, translates every file that needs it, and saves to zh/ subdirectory — all in one pass.
argument-hint: [目录，默认 /Volumes/six/MD]
---

## translate-md skill

Translate ALL English markdown files in a directory to Chinese in one pass.

### When to use
- "/translate-md"
- "/translate-md /some/dir"
- "翻译 markdown"
- "翻译这些 md 文件"

### How to execute

Run this ONE command. It handles everything — scanning, Chinese detection, and translation — via a shell loop that pipes each file to `claude -p`.

```bash
DIR="${1:-/Volumes/six/MD}"
OUT="$DIR/zh"
mkdir -p "$OUT"

FILES=$(python3 -c "
import os
direc = '$DIR'
files = []
for f in sorted(os.listdir(direc)):
    if not f.endswith('.md'):
        continue
    if os.path.exists(os.path.join('$OUT', f)):
        continue
    text = open(os.path.join(direc, f), 'r', errors='replace').read()
    if len(text) < 20:
        continue
    chinese = sum(1 for c in text if '一' <= c <= '鿿')
    if chinese / len(text) < 0.05:
        files.append(f)
print('\n'.join(files))
")

if [ -z "$FILES" ]; then
    echo "No files need translation"
    exit 0
fi

TOTAL=$(echo "$FILES" | wc -l | tr -d ' ')
echo "Translating $TOTAL files..."
echo ""

i=1
IFS=$'\n'
for f in $FILES; do
    echo "[$i/$TOTAL] $f"

    cat "$DIR/$f" | claude -p 'Translate this English markdown to Chinese. Respond with ONLY the translated markdown, no explanations. Keep all image references ![alt](path), links [text](url), code blocks, and markdown formatting exactly as-is. Preserve attachment/ paths unchanged.' > "$OUT/$f.tmp" 2>/dev/null

    if [ -s "$OUT/$f.tmp" ]; then
        sed -i '' -e '/^```/d' "$OUT/$f.tmp" 2>/dev/null || true
        mv "$OUT/$f.tmp" "$OUT/$f"
        echo "  done → $f"
    else
        echo "  FAILED $f"
        rm -f "$OUT/$f.tmp"
    fi

    i=$((i+1))
    sleep 1
done

echo ""
echo "Done → $OUT"
```

Do NOT read the files yourself. Do NOT translate incrementally turn-by-turn. Just run the single Bash command above.
