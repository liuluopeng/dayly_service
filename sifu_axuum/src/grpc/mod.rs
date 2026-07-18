//! gRPC 服务实现 — 剪贴板历史
//!
//! 由 proto/clipboard.proto 编译生成桩代码，在此实现业务逻辑。

pub mod clipboard {
    tonic::include_proto!("clipboard");
}

pub static CLIPBOARD_DESCRIPTOR: &[u8] =
    tonic::include_file_descriptor_set!("clipboard_descriptor");

pub static CLIPBOARD_SYNC_DESCRIPTOR: &[u8] =
    tonic::include_file_descriptor_set!("clipboard_sync_descriptor");

use clipboard::clipboard_history_server::{ClipboardHistory, ClipboardHistoryServer};
use clipboard::{HistoryEntry, HistoryRequest, HistoryResponse};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::FromRow;
use std::path::PathBuf;

/// 从 SQLite 行映射的记录（与 controller/clipboard.rs 共享结构）
#[derive(Debug, Clone, FromRow)]
struct Row {
    id: i64,
    entry_type: String,
    text_content: Option<String>,
    image_path: Option<String>,
    content_hash: String,
    created_at: String,
}

fn db_path() -> PathBuf {
    dirs::home_dir()
        .map(|p| p.join(".local-agent/history.db"))
        .unwrap_or_else(|| PathBuf::from("/tmp/.local-agent/history.db"))
}

async fn connect() -> Result<sqlx::SqlitePool, String> {
    let url = format!("sqlite://{}", db_path().display());
    SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await
        .map_err(|e| format!("连接数据库失败: {}", e))
}

async fn query_entries(
    count: i64,
    type_filter: Option<&str>,
    search: Option<&str>,
) -> Result<Vec<Row>, String> {
    let url = format!("sqlite://{}?mode=ro", db_path().display());
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await
        .map_err(|e| format!("连接数据库失败: {}", e))?;

    if let Some(keyword) = search {
        let pattern = format!("%{}%", keyword.replace('%', "\\%"));
        sqlx::query_as::<_, Row>(
            "SELECT id, entry_type, text_content, image_path, content_hash, created_at
             FROM clipboard_entries
             WHERE entry_type = 'text' AND text_content LIKE ?1
             ORDER BY created_at DESC LIMIT ?2",
        )
        .bind(&pattern)
        .bind(count)
        .fetch_all(&pool)
        .await
        .map_err(|e| format!("查询失败: {}", e))
    } else if let Some(ft) = type_filter {
        sqlx::query_as::<_, Row>(
            "SELECT id, entry_type, text_content, image_path, content_hash, created_at
             FROM clipboard_entries
             WHERE entry_type = ?1
             ORDER BY created_at DESC LIMIT ?2",
        )
        .bind(ft)
        .bind(count)
        .fetch_all(&pool)
        .await
        .map_err(|e| format!("查询失败: {}", e))
    } else {
        sqlx::query_as::<_, Row>(
            "SELECT id, entry_type, text_content, image_path, content_hash, created_at
             FROM clipboard_entries
             ORDER BY created_at DESC LIMIT ?1",
        )
        .bind(count)
        .fetch_all(&pool)
        .await
        .map_err(|e| format!("查询失败: {}", e))
    }
}

// ─── Service 实现 ──────────────────────────────────────────────

#[derive(Debug, Default)]
pub struct ClipboardHistorySvc;

#[tonic::async_trait]
impl ClipboardHistory for ClipboardHistorySvc {
    async fn get_history(
        &self,
        request: tonic::Request<HistoryRequest>,
    ) -> Result<tonic::Response<HistoryResponse>, tonic::Status> {
        let req = request.into_inner();
        let count = (req.count.max(1).min(200)) as i64;
        let type_filter = if req.type_filter.is_empty() {
            None
        } else {
            Some(req.type_filter.as_str())
        };
        let search = if req.search.is_empty() {
            None
        } else {
            Some(req.search.as_str())
        };

        match query_entries(count, type_filter, search).await {
            Ok(rows) => {
                let data: Vec<HistoryEntry> = rows
                    .into_iter()
                    .map(|r| {
                        let image_url = r.image_path.as_ref().and_then(|p| {
                            std::path::Path::new(p)
                                .file_name()
                                .map(|f| format!("/api/clipboard/images/{}", f.to_string_lossy()))
                        });
                        HistoryEntry {
                            id: r.id,
                            entry_type: r.entry_type,
                            text_content: r.text_content,
                            image_url,
                            image_path: r.image_path,
                            content_hash: r.content_hash,
                            created_at: r.created_at,
                        }
                    })
                    .collect();

                Ok(tonic::Response::new(HistoryResponse {
                    code: 200,
                    message: "ok".into(),
                    data,
                }))
            }
            Err(e) => Ok(tonic::Response::new(HistoryResponse {
                code: 500,
                message: e,
                data: vec![],
            })),
        }
    }
}

/// 构建 Tonic gRPC 服务，准备嵌入 axum
pub fn clipboard_grpc_service() -> ClipboardHistoryServer<ClipboardHistorySvc> {
    ClipboardHistoryServer::new(ClipboardHistorySvc)
}

// ─── Hello 服务 ────────────────────────────────────────────────

pub mod hello {
    tonic::include_proto!("hello");
}

pub static HELLO_DESCRIPTOR: &[u8] =
    tonic::include_file_descriptor_set!("hello_descriptor");

use hello::greeter_server::{Greeter, GreeterServer};
use hello::{HelloRequest, HelloResponse};

#[derive(Debug, Default)]
pub struct GreeterSvc;

#[tonic::async_trait]
impl Greeter for GreeterSvc {
    async fn say_hello(
        &self,
        request: tonic::Request<HelloRequest>,
    ) -> Result<tonic::Response<HelloResponse>, tonic::Status> {
        let name = request.into_inner().name;
        let message = if name.is_empty() {
            "Hello, World!".to_string()
        } else {
            format!("Hello, {}!", name)
        };
        Ok(tonic::Response::new(HelloResponse { message }))
    }
}

pub fn hello_grpc_service() -> GreeterServer<GreeterSvc> {
    GreeterServer::new(GreeterSvc)
}

// ─── ClipboardSync 服务 ────────────────────────────────────────
// local-agent 通过此接口推送剪贴板事件

pub mod clipboard_sync {
    tonic::include_proto!("clipboard_sync");
}

use clipboard_sync::clipboard_sync_server::{ClipboardSync, ClipboardSyncServer};
use clipboard_sync::{PushClipboardRequest, PushClipboardResponse};

async fn is_duplicate(pool: &sqlx::SqlitePool, hash: &str, entry_type: &str) -> Result<bool, String> {
    let result = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM clipboard_entries WHERE content_hash = ?1 AND entry_type = ?2",
    )
    .bind(hash)
    .bind(entry_type)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("去重查询失败: {}", e))?;

    Ok(result > 0)
}

#[derive(Debug, Default)]
pub struct ClipboardSyncSvc;

#[tonic::async_trait]
impl ClipboardSync for ClipboardSyncSvc {
    async fn push_clipboard(
        &self,
        request: tonic::Request<PushClipboardRequest>,
    ) -> Result<tonic::Response<PushClipboardResponse>, tonic::Status> {
        let req = request.into_inner();

        let pool = connect().await.map_err(|e| tonic::Status::internal(e))?;

        // 去重检查
        let dup = is_duplicate(&pool, &req.content_hash, &req.content_type)
            .await
            .map_err(|e| tonic::Status::internal(e))?;

        if dup {
            return Ok(tonic::Response::new(PushClipboardResponse {
                code: 200,
                message: "duplicate".into(),
                deduplicated: true,
            }));
        }

        match req.content_type.as_str() {
            "text" => {
                sqlx::query(
                    "INSERT INTO clipboard_entries (entry_type, text_content, image_path, content_hash, created_at)
                     VALUES (?1, ?2, NULL, ?3, ?4)",
                )
                .bind(&req.content_type)
                .bind(&req.text_content)
                .bind(&req.content_hash)
                .bind(&req.occurred_at)
                .execute(&pool)
                .await
                .map_err(|e| tonic::Status::internal(format!("写入失败: {}", e)))?;

                tracing::info!("📝 gRPC 收到剪贴板文本");
            }
            "image" => {
                sqlx::query(
                    "INSERT INTO clipboard_entries (entry_type, text_content, image_path, content_hash, created_at)
                     VALUES (?1, NULL, ?2, ?3, ?4)",
                )
                .bind(&req.content_type)
                .bind(&req.image_path)
                .bind(&req.content_hash)
                .bind(&req.occurred_at)
                .execute(&pool)
                .await
                .map_err(|e| tonic::Status::internal(format!("写入失败: {}", e)))?;

                tracing::info!("🖼️ gRPC 收到剪贴板图片: {}", req.image_path);
            }
            _ => {
                return Err(tonic::Status::invalid_argument(format!(
                    "未知类型: {}",
                    req.content_type
                )));
            }
        }

        Ok(tonic::Response::new(PushClipboardResponse {
            code: 200,
            message: "ok".into(),
            deduplicated: false,
        }))
    }
}

pub fn clipboard_sync_grpc_service() -> ClipboardSyncServer<ClipboardSyncSvc> {
    ClipboardSyncServer::new(ClipboardSyncSvc)
}
