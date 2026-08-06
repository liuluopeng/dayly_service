// 生成 hanzi 笔画 SVG 数据表（追加进 zici.db）
// 用法: node gen-hanzi-db.js <hanzi-writer-data目录> <zici.db路径>
const { DatabaseSync } = require('node:sqlite');
const fs = require('fs');
const path = require('path');

const dataDir = process.argv[2];
const dbPath = process.argv[3];

const db = new DatabaseSync(dbPath);
db.exec('CREATE TABLE IF NOT EXISTS hanzi_svg (char TEXT PRIMARY KEY, strokes TEXT)');

const files = fs.readdirSync(dataDir).filter((f) => f.endsWith('.json'));
const insert = db.prepare('INSERT OR REPLACE INTO hanzi_svg (char, strokes) VALUES (?, ?)');
db.exec('BEGIN');
let count = 0;
for (const f of files) {
  const char = f.replace('.json', '');
  if (char.length !== 1) continue;
  try {
    const d = JSON.parse(fs.readFileSync(path.join(dataDir, f), 'utf8'));
    if (!d.strokes || d.strokes.length === 0) continue;
    insert.run(char, JSON.stringify(d.strokes));
    count++;
  } catch {}
}
db.exec('COMMIT');
console.log(`hanzi_svg 表: ${count} 字`);
db.close();
