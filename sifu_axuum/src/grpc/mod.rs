use clipboard::clipboard_history_server::{ClipboardHistory, ClipboardHistoryServer};
use clipboard::{HistoryEntry, HistoryRequest, HistoryResponse};
use clipboard_sync::clipboard_sync_server::{ClipboardSync, ClipboardSyncServer};
use clipboard_sync::{PushClipboardRequest, PushClipboardResponse};
use sqlx::PgPool;

pub mod clipboard {
    tonic::include_proto!("clipboard");
}

pub static CLIPBOARD_DESCRIPTOR: &[u8] =
    tonic::include_file_descriptor_set!("clipboard_descriptor");

pub static CLIPBOARD_SYNC_DESCRIPTOR: &[u8] =
    tonic::include_file_descriptor_set!("clipboard_sync_descriptor");

pub mod hello {
    tonic::include_proto!("hello");
}

pub static HELLO_DESCRIPTOR: &[u8] =
    tonic::include_file_descriptor_set!("hello_descriptor");

use hello::greeter_server::{Greeter, GreeterServer};
use hello::{HelloRequest, HelloResponse};

pub mod clipboard_sync {
    tonic::include_proto!("clipboard_sync");
}

// ─── Hello 服务 ────────────────────────────────────────────────

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

// ─── ClipboardHistory 服务 ─────────────────────────────────────

pub struct ClipboardHistorySvc {
    pool: PgPool,
}

#[tonic::async_trait]
impl ClipboardHistory for ClipboardHistorySvc {
    async fn get_history(
        &self,
        request: tonic::Request<HistoryRequest>,
    ) -> Result<tonic::Response<HistoryResponse>, tonic::Status> {
        let req = request.into_inner();
        let count = (req.count.max(1).min(200)) as i64;

        let entries = if !req.search.is_empty() {
            let pattern = format!("%{}%", req.search.replace('%', "\\%"));
            sqlx::query_as::<_, ClipRow>(
                "SELECT id, entry_type, text_content, image_path, content_hash, created_at
                 FROM clipboard_entries
                 WHERE entry_type = 'text' AND text_content ILIKE $1
                 ORDER BY created_at DESC LIMIT $2",
            )
            .bind(&pattern)
            .bind(count)
            .fetch_all(&self.pool)
            .await
        } else if !req.type_filter.is_empty() {
            sqlx::query_as::<_, ClipRow>(
                "SELECT id, entry_type, text_content, image_path, content_hash, created_at
                 FROM clipboard_entries
                 WHERE entry_type = $1
                 ORDER BY created_at DESC LIMIT $2",
            )
            .bind(&req.type_filter)
            .bind(count)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, ClipRow>(
                "SELECT id, entry_type, text_content, image_path, content_hash, created_at
                 FROM clipboard_entries
                 ORDER BY created_at DESC LIMIT $1",
            )
            .bind(count)
            .fetch_all(&self.pool)
            .await
        };

        match entries {
            Ok(rows) => {
                let data: Vec<HistoryEntry> = rows
                    .into_iter()
                    .map(|r| HistoryEntry {
                        id: r.id,
                        entry_type: r.entry_type,
                        text_content: r.text_content,
                        image_url: r.image_path.as_ref().and_then(|p| {
                            std::path::Path::new(p)
                                .file_name()
                                .map(|f| format!("/api/clipboard/images/{}", f.to_string_lossy()))
                        }),
                        image_path: r.image_path,
                        content_hash: r.content_hash,
                        created_at: r.created_at.to_string(),
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
                message: e.to_string(),
                data: vec![],
            })),
        }
    }
}

pub fn clipboard_grpc_service(pool: PgPool) -> ClipboardHistoryServer<ClipboardHistorySvc> {
    ClipboardHistoryServer::new(ClipboardHistorySvc { pool })
}

// ─── ClipboardSync 服务 ────────────────────────────────────────

pub struct ClipboardSyncSvc {
    pool: PgPool,
}

#[tonic::async_trait]
impl ClipboardSync for ClipboardSyncSvc {
    async fn push_clipboard(
        &self,
        request: tonic::Request<PushClipboardRequest>,
    ) -> Result<tonic::Response<PushClipboardResponse>, tonic::Status> {
        let req = request.into_inner();

        // 去重检查
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM clipboard_entries WHERE content_hash = $1 AND entry_type = $2)",
        )
        .bind(&req.content_hash)
        .bind(&req.content_type)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(false);

        if exists {
            return Ok(tonic::Response::new(PushClipboardResponse {
                code: 200,
                message: "duplicate".into(),
                deduplicated: true,
            }));
        }

        let result = match req.content_type.as_str() {
            "text" => {
                sqlx::query(
                    "INSERT INTO clipboard_entries (entry_type, text_content, content_hash, created_at)
                     VALUES ($1, $2, $3, $4)",
                )
                .bind(&req.content_type)
                .bind(&req.text_content)
                .bind(&req.content_hash)
                .bind(&req.occurred_at)
                .execute(&self.pool)
                .await
            }
            "image" => {
                sqlx::query(
                    "INSERT INTO clipboard_entries (entry_type, image_path, content_hash, created_at)
                     VALUES ($1, $2, $3, $4)",
                )
                .bind(&req.content_type)
                .bind(&req.image_path)
                .bind(&req.content_hash)
                .bind(&req.occurred_at)
                .execute(&self.pool)
                .await
            }
            _ => return Err(tonic::Status::invalid_argument(format!("未知类型: {}", req.content_type))),
        };

        match result {
            Ok(_) => {
                tracing::info!("📋 剪贴板已存储: {}", req.content_type);
                Ok(tonic::Response::new(PushClipboardResponse {
                    code: 200,
                    message: "ok".into(),
                    deduplicated: false,
                }))
            }
            Err(e) => Err(tonic::Status::internal(format!("写入失败: {}", e))),
        }
    }
}

pub fn clipboard_sync_grpc_service(pool: PgPool) -> ClipboardSyncServer<ClipboardSyncSvc> {
    ClipboardSyncServer::new(ClipboardSyncSvc { pool })
}

#[derive(sqlx::FromRow)]
struct ClipRow {
    id: i64,
    entry_type: String,
    text_content: Option<String>,
    image_path: Option<String>,
    content_hash: String,
    created_at: chrono::DateTime<chrono::Utc>,
}
