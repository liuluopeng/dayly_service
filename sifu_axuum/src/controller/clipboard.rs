use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;
use std::path::PathBuf;
use tower_http::services::ServeDir;

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

async fn get_history() -> impl IntoResponse {
    Json(json!({"code": 200, "data": []}))
}
