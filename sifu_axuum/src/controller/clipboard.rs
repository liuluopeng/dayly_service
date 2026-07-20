use axum::extract::{Extension, Query};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use std::path::PathBuf;
use tower_http::services::ServeDir;

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    #[serde(default = "default_count")]
    count: usize,
    r#type: Option<String>,
    search: Option<String>,
}

fn default_count() -> usize { 20 }

pub fn clipboard_routes() -> Router {
    let image_dir = dirs::home_dir()
        .map(|p| p.join("Pictures/clipboard"))
        .unwrap_or_else(|| PathBuf::from("/tmp"));

    Router::new()
        .route("/history", get(get_history))
        .nest_service("/images", ServeDir::new(&image_dir).precompressed_gzip())
}

async fn get_history(
    Extension(pool): Extension<PgPool>,
    Query(params): Query<HistoryQuery>,
) -> impl IntoResponse {
    let count = (params.count.min(200) as i64).max(1);

    let entries = if let Some(ref keyword) = params.search {
        let pattern = format!("%{}%", keyword.replace('%', "\\%"));
        sqlx::query_as::<_, ClipRow>(
            "SELECT id, entry_type, text_content, image_path, content_hash, created_at
             FROM clipboard_entries
             WHERE entry_type = 'text' AND text_content ILIKE $1
             ORDER BY created_at DESC LIMIT $2",
        )
        .bind(&pattern)
        .bind(count)
        .fetch_all(&pool)
        .await
    } else if let Some(ref ft) = params.r#type {
        sqlx::query_as::<_, ClipRow>(
            "SELECT id, entry_type, text_content, image_path, content_hash, created_at
             FROM clipboard_entries
             WHERE entry_type = $1
             ORDER BY created_at DESC LIMIT $2",
        )
        .bind(ft)
        .bind(count)
        .fetch_all(&pool)
        .await
    } else {
        sqlx::query_as::<_, ClipRow>(
            "SELECT id, entry_type, text_content, image_path, content_hash, created_at
             FROM clipboard_entries
             ORDER BY created_at DESC LIMIT $1",
        )
        .bind(count)
        .fetch_all(&pool)
        .await
    };

    match entries {
        Ok(rows) => {
            let items: Vec<serde_json::Value> = rows.into_iter().map(|e| {
                let mut v = json!({
                    "id": e.id,
                    "type": e.entry_type,
                    "text_content": e.text_content,
                    "content_hash": e.content_hash,
                    "created_at": e.created_at.to_string(),
                });
                if let Some(ref path) = e.image_path {
                    let filename = std::path::Path::new(path)
                        .file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
                    v["image_url"] = json!(format!("/api/clipboard/images/{}", filename));
                    v["image_path"] = json!(path);
                }
                v
            }).collect();
            (axum::http::StatusCode::OK, Json(json!({"code": 200, "data": items})))
        }
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"code": 500, "message": format!("查询失败: {}", e)})),
        ),
    }
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
