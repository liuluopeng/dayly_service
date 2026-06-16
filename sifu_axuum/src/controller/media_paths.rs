use crate::middleware::Claims;
use axum::extract::{Extension, Path as AxumPath, Query};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use common::api::base::{ApiError, ApiResponse, ApiResult};
use my_type::model::media_paths::MediaPath;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ListMediaPathsQuery {
    pub media_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddMediaPathRequest {
    pub directory_id: Uuid,
    pub media_type: String,
    pub path: String,
    pub label: Option<String>,
}

// 当前用户：列出自己有权限的 media_paths
pub async fn list_media_paths(
    claims: Claims,
    Query(query): Query<ListMediaPathsQuery>,
    Extension(pg_pool): Extension<PgPool>,
) -> ApiResult<ApiResponse<Vec<MediaPath>>> {
    let username = &claims.username;

    let paths = if let Some(ref media_type) = query.media_type {
        sqlx::query_as::<_, MediaPath>(
            "SELECT id, directory_id, media_type, path, label, allow_list, scan_when_start, scan_when_change, last_scan_time, created_at
             FROM media_paths WHERE $1 = ANY(allow_list) AND media_type = $2 ORDER BY created_at",
        )
        .bind(username)
        .bind(media_type)
        .fetch_all(&pg_pool)
        .await
        .map_err(|e| ApiError::Internal(format!("查询媒体路径失败: {}", e)))?
    } else {
        sqlx::query_as::<_, MediaPath>(
            "SELECT id, directory_id, media_type, path, label, allow_list, scan_when_start, scan_when_change, last_scan_time, created_at
             FROM media_paths WHERE $1 = ANY(allow_list) ORDER BY created_at",
        )
        .bind(username)
        .fetch_all(&pg_pool)
        .await
        .map_err(|e| ApiError::Internal(format!("查询媒体路径失败: {}", e)))?
    };

    Ok(ApiResponse::ok(paths))
}

// 当前用户：添加 media_path
pub async fn add_media_path(
    claims: Claims,
    Extension(pg_pool): Extension<PgPool>,
    Json(body): Json<AddMediaPathRequest>,
) -> ApiResult<ApiResponse<MediaPath>> {
    let username = &claims.username;

    if body.path.is_empty() {
        return Err(ApiError::bad_request(ApiError::EMPTY_PATH, "路径不能为空"));
    }

    if !["song", "video", "photo", "book", "melatonin"].contains(&body.media_type.as_str()) {
        return Err(ApiError::bad_request(ApiError::INVALID_MEDIA_TYPE, "无效的媒体类型"));
    }

    // 验证当前用户在 directory 的 allow_list 中
    let dir_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM user_directories WHERE id = $1 AND $2 = ANY(allow_list))",
    )
    .bind(body.directory_id)
    .bind(username)
    .fetch_one(&pg_pool)
    .await
    .map_err(|e| ApiError::Internal(format!("验证目录失败: {}", e)))?;

    if !dir_exists {
        return Err(ApiError::not_found(ApiError::DIR_ACCESS_DENIED, "目录不存在或无权访问"));
    }

    // 数据库 trigger 会验证 path 必须是 directory 的子路径
    let media_path = sqlx::query_as::<_, MediaPath>(
        "INSERT INTO media_paths (id, directory_id, media_type, path, label, allow_list)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (path) DO UPDATE SET label = EXCLUDED.label, directory_id = EXCLUDED.directory_id, media_type = EXCLUDED.media_type, allow_list = EXCLUDED.allow_list
         RETURNING id, directory_id, media_type, path, label, allow_list, scan_when_start, scan_when_change, last_scan_time, created_at",
    )
    .bind(Uuid::now_v7())
    .bind(body.directory_id)
    .bind(&body.media_type)
    .bind(&body.path)
    .bind(&body.label.unwrap_or_default())
    .bind(&vec![username.clone()])
    .fetch_one(&pg_pool)
    .await
    .map_err(|e| ApiError::Internal(format!("添加媒体路径失败: {}", e)))?;

    Ok(ApiResponse::ok(media_path))
}

// 当前用户：删除 media_path
pub async fn delete_media_path(
    claims: Claims,
    Extension(pg_pool): Extension<PgPool>,
    AxumPath(path_id): AxumPath<Uuid>,
) -> ApiResult<ApiResponse<()>> {
    let username = &claims.username;

    let result = sqlx::query(
        "DELETE FROM media_paths WHERE id = $1 AND $2 = ANY(allow_list)",
    )
    .bind(path_id)
    .bind(username)
    .execute(&pg_pool)
    .await
    .map_err(|e| ApiError::Internal(format!("删除媒体路径失败: {}", e)))?;

    if result.rows_affected() == 0 {
        return Err(ApiError::not_found(ApiError::MEDIA_PATH_NOT_FOUND, "媒体路径不存在"));
    }

    Ok(ApiResponse::ok(()))
}

pub fn media_paths_routes() -> Router {
    Router::new()
        .route("/", get(list_media_paths).post(add_media_path))
        .route("/{id}", delete(delete_media_path))
}
