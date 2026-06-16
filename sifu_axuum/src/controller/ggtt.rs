use crate::middleware::Claims;
use axum::{
    Router,
    extract::{Extension, Json},
    http::StatusCode,
    routing::post,
};
use common::api::ggtt::SearchRequest;
use my_type::model::ggtt::GgttCode;
use sqlx::PgPool;
use tracing::error;

use common::api::base::{ApiError, ApiResponse, ApiResult};

pub async fn search_ggtt(
    claims: Claims,
    Json(req): Json<SearchRequest>,
) -> ApiResult<ApiResponse<GgttCode>> {
    let (code_86, svg1, svg2, svg3, svg4, has_diagram) = crate::controller::dict::lookup_ggtt_char(&req.search)
        .await
        .ok_or_else(|| ApiError::not_found(ApiError::CHARACTER_NOT_FOUND, "Character not found"))?;

    Ok(ApiResponse::ok(GgttCode {
        id: 0,
        char: req.search,
        code_86,
        has_diagram,
        svg1, svg2, svg3, svg4,
    }))
}

pub fn ggtt_routers() -> Router {
    Router::new().route("/search_ggtt", post(search_ggtt))
}
