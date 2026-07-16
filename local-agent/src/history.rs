//! clipboard-history — 剪贴板内容持久化存储（异步 SQLite）
//!
//! 使用 sqlx + SqlitePool 实现异步读写，支持去重、检索、自动淘汰。

use std::path::PathBuf;

use chrono::Local;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{FromRow, SqlitePool};

/// 一条剪贴板历史记录
#[derive(Debug, Clone, FromRow)]
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
    pool: SqlitePool,
    max_entries: usize,
}

impl ClipboardHistory {
    /// 打开（或创建）历史数据库
    pub async fn open(max_entries: usize) -> Result<Self, String> {
        let db_dir = db_dir();
        std::fs::create_dir_all(&db_dir)
            .map_err(|e| format!("创建数据目录失败: {}", e))?;

        let db_path = db_dir.join("history.db");
        // sqlite:///absolute/path 或 sqlite://相对路径
        let url = format!("sqlite://{}?mode=rwc", db_path.display());

        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(&url)
            .await
            .map_err(|e| format!("连接数据库失败 ({}): {}", db_path.display(), e))?;

        // 启用 WAL 模式
        sqlx::query("PRAGMA journal_mode=WAL;")
            .execute(&pool)
            .await
            .map_err(|e| format!("设置 WAL 失败: {}", e))?;

        // 建表
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS clipboard_entries (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                entry_type      TEXT NOT NULL CHECK(entry_type IN ('text','image')),
                text_content    TEXT,
                image_path      TEXT,
                content_hash    TEXT NOT NULL,
                created_at      TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .map_err(|e| format!("建表失败: {}", e))?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_entries_created
             ON clipboard_entries(created_at DESC)",
        )
        .execute(&pool)
        .await
        .map_err(|e| format!("建索引失败: {}", e))?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_entries_hash
             ON clipboard_entries(content_hash)",
        )
        .execute(&pool)
        .await
        .map_err(|e| format!("建哈希索引失败: {}", e))?;

        let hist = Self { pool, max_entries };
        hist.prune().await;
        Ok(hist)
    }

    /// 插入一条文本记录（自动去重）
    pub async fn insert_text(&self, text: &str, hash: &str) -> Result<bool, String> {
        if self.is_recent_duplicate("text", hash).await? {
            return Ok(false);
        }
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        sqlx::query(
            "INSERT INTO clipboard_entries (entry_type, text_content, content_hash, created_at)
             VALUES ('text', ?1, ?2, ?3)",
        )
        .bind(text)
        .bind(hash)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("写入文本记录失败: {}", e))?;

        self.prune().await;
        Ok(true)
    }

    /// 插入一条图片记录
    pub async fn insert_image(&self, image_path: &str, hash: &str) -> Result<bool, String> {
        if self.is_recent_duplicate("image", hash).await? {
            return Ok(false);
        }
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        sqlx::query(
            "INSERT INTO clipboard_entries (entry_type, image_path, content_hash, created_at)
             VALUES ('image', ?1, ?2, ?3)",
        )
        .bind(image_path)
        .bind(hash)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("写入图片记录失败: {}", e))?;

        self.prune().await;
        Ok(true)
    }

    /// 查询最近记录
    pub async fn recent(
        &self,
        count: usize,
        filter_type: Option<&str>,
    ) -> Result<Vec<HistoryEntry>, String> {
        let limit = count as i64;
        let rows = if let Some(ft) = filter_type {
            sqlx::query_as::<_, HistoryEntry>(
                "SELECT id, entry_type, text_content, image_path, content_hash, created_at
                 FROM clipboard_entries
                 WHERE entry_type = ?1
                 ORDER BY created_at DESC
                 LIMIT ?2",
            )
            .bind(ft)
            .bind(limit)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("查询失败: {}", e))?
        } else {
            sqlx::query_as::<_, HistoryEntry>(
                "SELECT id, entry_type, text_content, image_path, content_hash, created_at
                 FROM clipboard_entries
                 ORDER BY created_at DESC
                 LIMIT ?1",
            )
            .bind(limit)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("查询失败: {}", e))?
        };
        Ok(rows)
    }

    /// 搜索文本历史（LIKE 模糊匹配）
    pub async fn search(
        &self,
        keyword: &str,
        count: usize,
    ) -> Result<Vec<HistoryEntry>, String> {
        let pattern = format!("%{}%", keyword.replace('%', "\\%"));
        let rows = sqlx::query_as::<_, HistoryEntry>(
            "SELECT id, entry_type, text_content, image_path, content_hash, created_at
             FROM clipboard_entries
             WHERE entry_type = 'text' AND text_content LIKE ?1
             ORDER BY created_at DESC
             LIMIT ?2",
        )
        .bind(&pattern)
        .bind(count as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("搜索失败: {}", e))?;
        Ok(rows)
    }

    /// 获取总条目数
    pub async fn total_count(&self) -> Result<i64, String> {
        let (count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM clipboard_entries",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("计数失败: {}", e))?;
        Ok(count)
    }

    // ─── 内部方法 ───────────────────────────────────────────────

    /// 检查是否与同类型的最近一条记录哈希相同
    async fn is_recent_duplicate(&self, entry_type: &str, hash: &str) -> Result<bool, String> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT content_hash FROM clipboard_entries
             WHERE entry_type = ?1
             ORDER BY created_at DESC LIMIT 1",
        )
        .bind(entry_type)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("去重查询失败: {}", e))?;

        match row {
            Some((last_hash,)) => Ok(last_hash == hash),
            None => Ok(false),
        }
    }

    /// 淘汰超出上限的旧记录
    async fn prune(&self) {
        let Ok((count,)) = sqlx::query_as::<_, (i64,)>(
            "SELECT COUNT(*) FROM clipboard_entries",
        )
        .fetch_one(&self.pool)
        .await
        else {
            return;
        };

        if count > self.max_entries as i64 {
            let keep = self.max_entries as i64;
            // 子查询删除最旧的 N 条
            let _ = sqlx::query(
                "DELETE FROM clipboard_entries WHERE id IN (
                    SELECT id FROM clipboard_entries
                    ORDER BY created_at DESC
                    LIMIT -1 OFFSET ?1
                )",
            )
            .bind(keep)
            .execute(&self.pool)
            .await;
        }
    }
}

fn db_dir() -> PathBuf {
    dirs::home_dir()
        .map(|p| p.join(".local-agent"))
        .unwrap_or_else(|| PathBuf::from("/tmp/.local-agent"))
}
