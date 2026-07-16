//! clipboard-history — 剪贴板内容持久化存储
//!
//! 使用 SQLite 存储文本/图片剪贴板记录，支持去重、检索、自动淘汰。

use std::path::PathBuf;

use chrono::Local;
use rusqlite::{params, Connection, Result as SqlResult};

/// 一条剪贴板历史记录
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub id: i64,
    pub entry_type: String, // "text" | "image"
    pub text_content: Option<String>,
    pub image_path: Option<String>,
    pub content_hash: String,
    pub created_at: String,
}

/// 剪贴板历史管理器
pub struct ClipboardHistory {
    conn: Connection,
    max_entries: usize,
}

impl ClipboardHistory {
    /// 打开（或创建）历史数据库
    pub fn open(max_entries: usize) -> Result<Self, String> {
        let db_dir = db_dir();
        std::fs::create_dir_all(&db_dir).map_err(|e| format!("创建数据目录失败: {}", e))?;

        let db_path = db_dir.join("history.db");
        let conn =
            Connection::open(&db_path).map_err(|e| format!("打开数据库失败 ({}): {}", db_path.display(), e))?;

        // 启用 WAL 模式（读写不阻塞）
        conn.execute_batch("PRAGMA journal_mode=WAL;")
            .map_err(|e| format!("设置 WAL 失败: {}", e))?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS clipboard_entries (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                entry_type      TEXT NOT NULL CHECK(entry_type IN ('text','image')),
                text_content    TEXT,
                image_path      TEXT,
                content_hash    TEXT NOT NULL,
                created_at      TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_entries_created
                ON clipboard_entries(created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_entries_hash
                ON clipboard_entries(content_hash);",
        )
        .map_err(|e| format!("初始化数据库表失败: {}", e))?;

        let hist = Self { conn, max_entries };
        hist.prune()?;
        Ok(hist)
    }

    /// 插入一条文本记录（自动去重：与最近一条文本哈希相同则跳过）
    pub fn insert_text(&self, text: &str, hash: &str) -> Result<bool, String> {
        // 去重检查
        if self.is_recent_duplicate("text", hash)? {
            return Ok(false);
        }
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        self.conn
            .execute(
                "INSERT INTO clipboard_entries (entry_type, text_content, content_hash, created_at)
                 VALUES ('text', ?1, ?2, ?3)",
                params![text, hash, now],
            )
            .map_err(|e| format!("写入文本记录失败: {}", e))?;

        // 写后异步 prune（不阻塞）
        let _ = self.conn.execute_batch("BEGIN;");
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM clipboard_entries", [], |r| r.get(0))
            .unwrap_or(0);
        if count > self.max_entries as i64 {
            let keep = self.max_entries as i64;
            self.conn
                .execute(
                    "DELETE FROM clipboard_entries WHERE id <= (
                        SELECT id FROM clipboard_entries ORDER BY created_at DESC LIMIT 1 OFFSET ?1
                    )",
                    params![keep],
                )
                .ok();
        }
        let _ = self.conn.execute_batch("COMMIT;");

        Ok(true)
    }

    /// 插入一条图片记录
    pub fn insert_image(&self, image_path: &str, hash: &str) -> Result<bool, String> {
        if self.is_recent_duplicate("image", hash)? {
            return Ok(false);
        }
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        self.conn
            .execute(
                "INSERT INTO clipboard_entries (entry_type, image_path, content_hash, created_at)
                 VALUES ('image', ?1, ?2, ?3)",
                params![image_path, hash, now],
            )
            .map_err(|e| format!("写入图片记录失败: {}", e))?;
        Ok(true)
    }

    /// 查询最近记录
    pub fn recent(&self, count: usize, filter_type: Option<&str>) -> Result<Vec<HistoryEntry>, String> {
        let query = match filter_type {
            Some(t) => format!(
                "SELECT id, entry_type, text_content, image_path, content_hash, created_at
                 FROM clipboard_entries WHERE entry_type = '{}'
                 ORDER BY created_at DESC LIMIT {}",
                t.replace('\'', "''"),
                count
            ),
            None => format!(
                "SELECT id, entry_type, text_content, image_path, content_hash, created_at
                 FROM clipboard_entries ORDER BY created_at DESC LIMIT {}",
                count
            ),
        };

        let mut stmt = self
            .conn
            .prepare(&query)
            .map_err(|e| format!("查询准备失败: {}", e))?;

        let rows = stmt
            .query_map([], |r| {
                Ok(HistoryEntry {
                    id: r.get(0)?,
                    entry_type: r.get(1)?,
                    text_content: r.get(2)?,
                    image_path: r.get(3)?,
                    content_hash: r.get(4)?,
                    created_at: r.get(5)?,
                })
            })
            .map_err(|e| format!("查询执行失败: {}", e))?;

        let mut entries = Vec::new();
        for row in rows {
            entries.push(row.map_err(|e| format!("读取记录失败: {}", e))?);
        }
        Ok(entries)
    }

    /// 搜索文本历史
    pub fn search(&self, keyword: &str, count: usize) -> Result<Vec<HistoryEntry>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, entry_type, text_content, image_path, content_hash, created_at
                 FROM clipboard_entries
                 WHERE entry_type = 'text' AND text_content LIKE ?1
                 ORDER BY created_at DESC
                 LIMIT ?2",
            )
            .map_err(|e| format!("搜索准备失败: {}", e))?;

        let pattern = format!("%{}%", keyword.replace('%', "\\%"));
        let rows = stmt
            .query_map(params![pattern, count as i64], |r| {
                Ok(HistoryEntry {
                    id: r.get(0)?,
                    entry_type: r.get(1)?,
                    text_content: r.get(2)?,
                    image_path: r.get(3)?,
                    content_hash: r.get(4)?,
                    created_at: r.get(5)?,
                })
            })
            .map_err(|e| format!("搜索执行失败: {}", e))?;

        let mut entries = Vec::new();
        for row in rows {
            entries.push(row.map_err(|e| format!("读取搜索结果失败: {}", e))?);
        }
        Ok(entries)
    }

    /// 获取总条目数
    pub fn total_count(&self) -> Result<i64, String> {
        self.conn
            .query_row("SELECT COUNT(*) FROM clipboard_entries", [], |r| r.get(0))
            .map_err(|e| format!("计数失败: {}", e))
    }

    // ─── 内部方法 ───────────────────────────────────────────────

    /// 检查是否与同类型的最近一条记录哈希相同（去重）
    fn is_recent_duplicate(&self, entry_type: &str, hash: &str) -> Result<bool, String> {
        let result: SqlResult<String> = self.conn.query_row(
            "SELECT content_hash FROM clipboard_entries
             WHERE entry_type = ?1
             ORDER BY created_at DESC LIMIT 1",
            params![entry_type],
            |r| r.get(0),
        );
        match result {
            Ok(last) => Ok(last == hash),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(false),
            Err(e) => Err(format!("去重查询失败: {}", e)),
        }
    }

    fn prune(&self) -> Result<(), String> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM clipboard_entries", [], |r| r.get(0))
            .map_err(|e| format!("计数失败: {}", e))?;

        if count > self.max_entries as i64 {
            let keep = self.max_entries as i64;
            self.conn
                .execute(
                    "DELETE FROM clipboard_entries WHERE id IN (
                        SELECT id FROM clipboard_entries ORDER BY created_at DESC LIMIT -1 OFFSET ?1
                    )",
                    params![keep],
                )
                .map_err(|e| format!("淘汰旧记录失败: {}", e))?;
        }
        Ok(())
    }
}

fn db_dir() -> PathBuf {
    dirs::home_dir()
        .map(|p| p.join(".local-agent"))
        .unwrap_or_else(|| PathBuf::from("/tmp/.local-agent"))
}
