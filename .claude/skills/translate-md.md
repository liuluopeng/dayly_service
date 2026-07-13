---
name: translate-md
description: Translate English markdown files to Chinese. Automatically detects language and skips already-Chinese files.
argument-hint: [目录，默认 /Volumes/six/MD]
---

## translate-md skill

Translate English markdown files in a directory to Chinese.

### When to use
- "/translate-md"
- "翻译 markdown"
- "翻译这些 md 文件"
- "把英文的 .md 翻译成中文"

### How to execute

1. **Determine the directory** — default `/Volumes/six/MD`, or from user input.

2. **Scan for files needing translation** — use this detection (python3 handles Unicode):

```bash
python3 -c "
import os, sys
direc = sys.argv[1] if len(sys.argv) > 1 else '/Volumes/six/MD'
for f in sorted(os.listdir(direc)):
    if f.endswith('.md') and not os.path.exists(os.path.join(direc, 'zh', f)):
        text = open(os.path.join(direc, f), 'r', errors='replace').read()
        chinese = sum(1 for c in text if '一' <= c <= '鿿')
        if len(text) > 20 and chinese / len(text) < 0.05:
            print(f)
" "$1"
```

3. **Translate each file one by one.** For each file:
   - Read it with the Read tool
   - Translate the content from English to Chinese
   - Keep ALL markdown formatting, image references (`![alt](path)`, `[text](url)`), code blocks EXACTLY as-is
   - Write the translated file to `{input_dir}/zh/{basename}`
   - Then move to the next file

4. **Important translation rules:**
   - Output ONLY the translated markdown — no greetings, explanations, or notes
   - Preserve all `attachment/` paths unchanged
   - Keep YAML frontmatter if present
   - Translate the article title and body, but not URLs or file paths

### Example

```
/translate-md
→ Scan /Volumes/six/MD
→ Found 3 files needing translation
→ Translating file 1/3: a.md
→ Translating file 2/3: b.md
→ Done: 3 translated → /Volumes/six/MD/zh/
```
