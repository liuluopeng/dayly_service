//! clipboard — 通过 HTTP 访问剪贴板历史（读取 local-agent 的 SQLite 数据库）

use axum::extract::Query;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::FromRow;
use std::path::PathBuf;
use tower_http::services::ServeDir;

/// 历史记录的查询参数
#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    #[serde(default = "default_count")]
    count: usize,
    /// 过滤类型: text / image
    r#type: Option<String>,
    /// 搜索关键字
    search: Option<String>,
}

fn default_count() -> usize {
    20
}

/// 从 SQLite 行映射的记录
#[derive(Debug, Clone, FromRow)]
struct HistoryEntry {
    id: i64,
    entry_type: String,
    text_content: Option<String>,
    image_path: Option<String>,
    content_hash: String,
    created_at: String,
}

fn db_path() -> PathBuf {
    std::env::var("CLIPBOARD_DB")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .map(|p| p.join(".local-agent/history.db"))
                .unwrap_or_else(|| PathBuf::from("/app/data/clipboard.db"))
        })
}

/// 每次请求开一个连接池（单连接，WAL 模式廉价的本地操作）
async fn connect() -> Result<sqlx::SqlitePool, String> {
    let url = format!("sqlite://{}?mode=ro", db_path().display());
    SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await
        .map_err(|e| format!("连接剪贴板数据库失败: {}", e))
}

// ─── 路由 ──────────────────────────────────────────────────────

pub fn clipboard_routes() -> Router {
    let image_dir = dirs::home_dir()
        .map(|p| p.join("Pictures/clipboard"))
        .unwrap_or_else(|| PathBuf::from("/tmp"));

    Router::new()
        .route("/history", get(get_history))
        .nest_service(
            "/images",
            ServeDir::new(&image_dir).precompressed_gzip(),
        )
}

// ─── handlers ──────────────────────────────────────────────────

/// GET /api/clipboard/history?count=20&type=text&search=xxx
async fn get_history(Query(params): Query<HistoryQuery>) -> impl IntoResponse {
    let pool = match connect().await {
        Ok(p) => p,
        Err(e) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "code": 500,
                    "message": e,
                })),
            );
        }
    };

    let count = (params.count.min(200) as i64).max(1);

    let entries: Result<Vec<HistoryEntry>, _> = if let Some(ref keyword) = params.search {
        let pattern = format!("%{}%", keyword.replace('%', "\\%"));
        sqlx::query_as::<_, HistoryEntry>(
            "SELECT id, entry_type, text_content, image_path, content_hash, created_at
             FROM clipboard_entries
             WHERE entry_type = 'text' AND text_content LIKE ?1
             ORDER BY created_at DESC
             LIMIT ?2",
        )
        .bind(&pattern)
        .bind(count)
        .fetch_all(&pool)
        .await
    } else if let Some(ref ft) = params.r#type {
        sqlx::query_as::<_, HistoryEntry>(
            "SELECT id, entry_type, text_content, image_path, content_hash, created_at
             FROM clipboard_entries
             WHERE entry_type = ?1
             ORDER BY created_at DESC
             LIMIT ?2",
        )
        .bind(ft)
        .bind(count)
        .fetch_all(&pool)
        .await
    } else {
        sqlx::query_as::<_, HistoryEntry>(
            "SELECT id, entry_type, text_content, image_path, content_hash, created_at
             FROM clipboard_entries
             ORDER BY created_at DESC
             LIMIT ?1",
        )
        .bind(count)
        .fetch_all(&pool)
        .await
    };

    match entries {
        Ok(entries) => {
            let items: Vec<serde_json::Value> = entries
                .into_iter()
                .map(|e| {
                    let mut v = json!({
                        "id": e.id,
                        "type": e.entry_type,
                        "text_content": e.text_content,
                        "content_hash": e.content_hash,
                        "created_at": e.created_at,
                    });
                    if let Some(ref path) = e.image_path {
                        let filename = std::path::Path::new(path)
                            .file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_default();
                        v["image_url"] = json!(format!("/api/clipboard/images/{}", filename));
                        v["image_path"] = json!(path);
                    }
                    v
                })
                .collect();

            (axum::http::StatusCode::OK, Json(json!({"code": 200, "data": items})))
        }
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "code": 500,
                "message": format!("查询失败: {}", e),
            })),
        ),
    }
}
