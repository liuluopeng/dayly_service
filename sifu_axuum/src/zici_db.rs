//! zici 字词学习数据——SQLite 存储（生字/生词/词频）
//! 与词典 dict.db 同款方案：宿主机预生成 cold_data/zici.db，
//! 生产环境由 volume 挂载进容器（docker-compose 已挂 ./cold_data:/app/cold_data），
//! 容器内只读查询；开发环境缺失时从 data/*.json 自动生成。

use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;

static DB: Mutex<Option<Connection>> = Mutex::new(None);

pub fn ensure_zici_db() {
    let path = std::env::current_dir()
        .unwrap_or_default()
        .join("cold_data/zici.db");
    if !path.exists() {
        match generate(&path) {
            Ok(()) => tracing::info!("zici.db 已生成: {}", path.display()),
            Err(e) => tracing::error!(
                "zici.db 不存在且生成失败（{}）——生产环境请先运行 scripts/gen-zici-db.sh 预生成",
                e
            ),
        }
    }
    if let Ok(conn) = Connection::open(&path) {
        *DB.lock().unwrap() = Some(conn);
    } else {
        tracing::error!("无法打开 zici.db: {}", path.display());
    }
}

/// 独立生成入口（scripts/gen-zici-db.sh → src/bin/gen_zici_db.rs 调用）
pub fn generate_db(target: &str) -> Result<(), String> {
    let path = Path::new(target);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    generate(path)
}

fn generate(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let mut conn = Connection::open(path).map_err(|e| e.to_string())?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS zici_chars (grade INTEGER, term INTEGER, chars TEXT);
         CREATE TABLE IF NOT EXISTS zici_words (id INTEGER PRIMARY KEY AUTOINCREMENT, word TEXT UNIQUE);
         CREATE TABLE IF NOT EXISTS word_frequency (id INTEGER PRIMARY KEY AUTOINCREMENT, word TEXT UNIQUE, pinyin TEXT, frequency INTEGER, explanation TEXT);
         CREATE INDEX IF NOT EXISTS idx_wf_word ON word_frequency(word);
         CREATE TABLE IF NOT EXISTS pinyin_initial (pinyin TEXT PRIMARY KEY, char TEXT, example TEXT);
         CREATE TABLE IF NOT EXISTS pinyin_final (pinyin TEXT PRIMARY KEY, char TEXT, example TEXT);
         CREATE TABLE IF NOT EXISTS pinyin_combo (initial TEXT, final TEXT, combo TEXT, PRIMARY KEY (initial, final));",
    )
    .map_err(|e| e.to_string())?;

    let data_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("data");

    // 生字
    let chars_json =
        std::fs::read_to_string(data_dir.join("zici_chars.json")).map_err(|e| e.to_string())?;
    let chars: Vec<serde_json::Value> =
        serde_json::from_str(&chars_json).map_err(|e| e.to_string())?;
    {
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        for item in &chars {
            let grade = item["grade"].as_i64().unwrap_or(1);
            let term = item["term"].as_i64().unwrap_or(1);
            let chars = item["chars"].as_str().unwrap_or("");
            tx.execute(
                "INSERT OR IGNORE INTO zici_chars (grade, term, chars) VALUES (?1, ?2, ?3)",
                rusqlite::params![grade, term, chars],
            )
            .map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())?;
    }

    // 生词
    let words_json =
        std::fs::read_to_string(data_dir.join("zici_words.json")).map_err(|e| e.to_string())?;
    let words: Vec<String> = serde_json::from_str(&words_json).map_err(|e| e.to_string())?;
    {
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        for w in &words {
            tx.execute(
                "INSERT OR IGNORE INTO zici_words (word) VALUES (?1)",
                rusqlite::params![w],
            )
            .map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())?;
    }

    // 拼音声母/韵母/组合
    let pinyin_json =
        std::fs::read_to_string(data_dir.join("pinyin_combos.json")).map_err(|e| e.to_string())?;
    let pinyin_data: serde_json::Value =
        serde_json::from_str(&pinyin_json).map_err(|e| e.to_string())?;
    {
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        for item in pinyin_data["initials"].as_array().unwrap_or(&vec![]) {
            let pinyin = item["pinyin"].as_str().unwrap_or("");
            let char = item["char"].as_str().unwrap_or("");
            let example = item["example"].as_str().unwrap_or("");
            tx.execute(
                "INSERT OR IGNORE INTO pinyin_initial (pinyin, char, example) VALUES (?1, ?2, ?3)",
                rusqlite::params![pinyin, char, example],
            )
            .map_err(|e| e.to_string())?;
        }
        for item in pinyin_data["finals"].as_array().unwrap_or(&vec![]) {
            let pinyin = item["pinyin"].as_str().unwrap_or("");
            let char = item["char"].as_str().unwrap_or("");
            let example = item["example"].as_str().unwrap_or("");
            tx.execute(
                "INSERT OR IGNORE INTO pinyin_final (pinyin, char, example) VALUES (?1, ?2, ?3)",
                rusqlite::params![pinyin, char, example],
            )
            .map_err(|e| e.to_string())?;
        }
        if let Some(combos) = pinyin_data["combos"].as_object() {
            for (initial, finals_map) in combos {
                if let Some(finals_map) = finals_map.as_object() {
                    for (final_py, combo) in finals_map {
                        tx.execute(
                            "INSERT OR IGNORE INTO pinyin_combo (initial, final, combo) VALUES (?1, ?2, ?3)",
                            rusqlite::params![initial, final_py, combo.as_str().unwrap_or("")],
                        )
                        .map_err(|e| e.to_string())?;
                    }
                }
            }
        }
        tx.commit().map_err(|e| e.to_string())?;
    }

    // 词频（56000 条，含释义）
    let freq_json = std::fs::read_to_string(data_dir.join("word_frequency_list.json"))
        .map_err(|e| e.to_string())?;
    let freq: Vec<serde_json::Value> =
        serde_json::from_str(&freq_json).map_err(|e| e.to_string())?;
    {
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        {
            let mut stmt = tx
                .prepare(
                    "INSERT OR IGNORE INTO word_frequency (word, pinyin, frequency, explanation) VALUES (?1, ?2, ?3, ?4)",
                )
                .map_err(|e| e.to_string())?;
            for item in &freq {
                let word = item["word"].as_str().unwrap_or("");
                if word.is_empty() {
                    continue;
                }
                stmt.execute(rusqlite::params![
                    word,
                    item["pinyin_flat"].as_str().unwrap_or(""),
                    item["frequency"].as_u64().unwrap_or(0),
                    item["explanation"].as_str().unwrap_or(""),
                ])
                .map_err(|e| e.to_string())?;
            }
        }
        tx.commit().map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn with_db<T>(f: impl FnOnce(&Connection) -> T) -> T {
    let guard = DB.lock().unwrap();
    let conn = guard.as_ref().expect("zici.db 未初始化");
    f(conn)
}

/// 生字表：按年级/学期（grade 1-6，term 1-2）
pub fn zici_chars(grade: i64, term: i64) -> String {
    with_db(|conn| {
        conn.query_row(
            "SELECT chars FROM zici_chars WHERE grade = ?1 AND term = ?2",
            rusqlite::params![grade, term],
            |row| row.get::<_, String>(0),
        )
        .unwrap_or_default()
    })
}

/// 生词表：搜索 + 分页
pub fn zici_words(search: &str, page: i64, page_size: i64) -> (Vec<String>, i64) {
    with_db(|conn| {
        let like = format!("%{}%", search);
        let total: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM zici_words WHERE (?1 = '' OR word LIKE ?2)",
                rusqlite::params![search, like],
                |row| row.get(0),
            )
            .unwrap_or(0);
        let offset = (page - 1).max(0) * page_size;
        let mut stmt = conn
            .prepare(
                "SELECT word FROM zici_words WHERE (?1 = '' OR word LIKE ?2) ORDER BY id LIMIT ?3 OFFSET ?4",
            )
            .unwrap();
        let words: Vec<String> = stmt
            .query_map(rusqlite::params![search, like, page_size, offset], |row| {
                row.get(0)
            })
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        (words, total)
    })
}

/// 词频搜索（含释义）
pub fn zici_word_frequency(search: &str, limit: i64) -> Vec<(String, String, i64, String)> {
    with_db(|conn| {
        let like = format!("%{}%", search);
        let mut stmt = conn
            .prepare(
                "SELECT word, pinyin, frequency, explanation FROM word_frequency \
                 WHERE (?1 = '' OR word LIKE ?2) ORDER BY frequency DESC LIMIT ?3",
            )
            .unwrap();
        stmt.query_map(rusqlite::params![search, like, limit], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    })
}

/// 汉字笔画 SVG：返回 strokes JSON（每笔一个 SVG path）
pub fn hanzi_svg(char: &str) -> Option<String> {
    with_db(|conn| {
        conn.query_row(
            "SELECT strokes FROM hanzi_svg WHERE char = ?1",
            rusqlite::params![char],
            |row| row.get::<_, String>(0),
        )
        .ok()
    })
}

/// 拼音声母/韵母/合法组合（拼音组合学习）
pub fn pinyin_combos() -> (
    Vec<(String, String, String)>,
    Vec<(String, String, String)>,
    Vec<(String, String, String)>,
) {
    with_db(|conn| {
        let initials = {
            let mut stmt = conn
                .prepare("SELECT pinyin, char, example FROM pinyin_initial")
                .unwrap();
            stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect()
        };
        let finals = {
            let mut stmt = conn
                .prepare("SELECT pinyin, char, example FROM pinyin_final")
                .unwrap();
            stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect()
        };
        let combos = {
            let mut stmt = conn
                .prepare("SELECT initial, final, combo FROM pinyin_combo")
                .unwrap();
            stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect()
        };
        (initials, finals, combos)
    })
}
