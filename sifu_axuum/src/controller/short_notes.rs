use crate::middleware::Claims;
use axum::Router;
use axum::extract::{Extension, Json, Path, Query};
use axum::routing::{delete, get, post, put};
use chrono::Local;
use common::api::base::{ApiError, ApiResponse, ApiResult};
use my_type::model::short_notes::ShortNote;
use serde::Deserialize;
use sqlx::PgPool;
use tracing::error;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateShortNoteRequest {
    pub content: Option<String>,
    pub view_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateShortNoteRequest {
    pub content: Option<String>,
    pub view_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
}

fn default_page() -> u32 {
    1
}

fn default_page_size() -> u32 {
    10
}

pub async fn list_short_notes(
    Extension(pool): Extension<PgPool>,
    claims: Claims,
    Query(query): Query<ListQuery>,
) -> ApiResult<ApiResponse<Vec<ShortNote>>> {
    let offset = ((query.page.saturating_sub(1)) * query.page_size) as i64;
    let limit = query.page_size as i64;

    let user_id = Uuid::parse_str(&claims.id).unwrap_or_default();
    let records = sqlx::query_as::<_, ShortNote>(
        "SELECT id, content, view_id, view_name, created_at, user_id FROM short_notes WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
    )
    .bind(user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        error!("Database query error: {:?}", e);
        ApiError::Internal(e.to_string())
    })?;

    Ok(ApiResponse::ok(records))
}

pub async fn get_short_note(
    Extension(pool): Extension<PgPool>,
    claims: Claims,
    Path(id): Path<Uuid>,
) -> ApiResult<ApiResponse<ShortNote>> {
    let user_id = Uuid::parse_str(&claims.id).unwrap_or_default();
    let record = sqlx::query_as::<_, ShortNote>(
        "SELECT id, content, view_id, view_name, created_at, user_id FROM short_notes WHERE id = $1 AND user_id = $2",
    )
    .bind(id)
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        error!("Database query error: {:?}", e);
        match e {
            sqlx::Error::RowNotFound => ApiError::not_found(ApiError::SHORT_NOTE_NOT_FOUND, "Short note not found"),
            _ => ApiError::Internal(e.to_string()),
        }
    })?;

    Ok(ApiResponse::ok(record))
}

pub async fn create_short_note(
    Extension(pool): Extension<PgPool>,
    claims: Claims,
    Json(req): Json<CreateShortNoteRequest>,
) -> ApiResult<ApiResponse<ShortNote>> {
    let id = Uuid::new_v4();
    let created_at = Local::now();

    let user_id = Uuid::parse_str(&claims.id).unwrap_or_default();
    let record = sqlx::query_as::<_, ShortNote>(
        "INSERT INTO short_notes (id, content, view_name, created_at, user_id) VALUES ($1, $2, $3, $4, $5) RETURNING id, content, view_id, view_name, created_at, user_id"
    )
    .bind(id)
    .bind(&req.content)
    .bind(&req.view_name)
    .bind(created_at)
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        error!("Database insert error: {:?}", e);
        ApiError::Internal(e.to_string())
    })?;

    Ok(ApiResponse::ok(record))
}

pub async fn update_short_note(
    Extension(pool): Extension<PgPool>,
    claims: Claims,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateShortNoteRequest>,
) -> ApiResult<ApiResponse<ShortNote>> {
    let user_id = Uuid::parse_str(&claims.id).unwrap_or_default();
    let record = sqlx::query_as::<_, ShortNote>(
        "UPDATE short_notes SET content = COALESCE($1, content), view_name = COALESCE($2, view_name) WHERE id = $3 AND user_id = $4 RETURNING id, content, view_id, view_name, created_at, user_id"
    )
    .bind(&req.content)
    .bind(&req.view_name)
    .bind(id)
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        error!("Database update error: {:?}", e);
        match e {
            sqlx::Error::RowNotFound => ApiError::not_found(ApiError::SHORT_NOTE_NOT_FOUND, "Short note not found"),
            _ => ApiError::Internal(e.to_string()),
        }
    })?;

    Ok(ApiResponse::ok(record))
}

pub async fn delete_short_note(
    Extension(pool): Extension<PgPool>,
    claims: Claims,
    Path(id): Path<Uuid>,
) -> ApiResult<ApiResponse<()>> {
    let user_id = Uuid::parse_str(&claims.id).unwrap_or_default();
    let result = sqlx::query("DELETE FROM short_notes WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(&pool)
        .await
        .map_err(|e| {
            error!("Database delete error: {:?}", e);
            ApiError::Internal(e.to_string())
        })?;

    if result.rows_affected() == 0 {
        return Err(ApiError::not_found(ApiError::SHORT_NOTE_NOT_FOUND, "Short note not found"));
    }

    Ok(ApiResponse::ok(()))
}

pub fn short_notes_routes() -> Router {
    Router::new()
        .route("/list", get(list_short_notes))
        .route("/get/{id}", get(get_short_note))
        .route("/create", post(create_short_note))
        .route("/update/{id}", put(update_short_note))
        .route("/delete/{id}", delete(delete_short_note))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;
    use axum::Json;

    #[tokio::test]
    async fn test_create_and_get_short_note() {
        let pool = test_pool().await;
        let user_id = create_test_user(&pool, "test_sn_user").await;
        let claims = test_claims(user_id, false);

        // 创建
        let req = CreateShortNoteRequest {
            content: Some("测试内容".to_string()),
            view_name: Some("测试视图".to_string()),
        };
        let result = create_short_note(Extension(pool.clone()), claims.clone(), Json(req)).await;
        if let Err(ref e) = result {
            panic!("create_short_note 失败: {:?}", e);
        }
        assert!(result.is_ok());
        let note = result.unwrap().data.unwrap();
        assert_eq!(note.content.as_deref(), Some("测试内容"));
        assert_eq!(note.view_name.as_deref(), Some("测试视图"));

        // 获取
        let result2 = get_short_note(Extension(pool.clone()), claims, Path(note.id)).await;
        assert!(result2.is_ok());
        let fetched = result2.unwrap().data.unwrap();
        assert_eq!(fetched.id, note.id);

        // 清理
        sqlx::query("DELETE FROM short_notes WHERE user_id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .ok();
        cleanup_test_user(&pool, user_id).await;
    }

    #[tokio::test]
    async fn test_list_short_notes() {
        let pool = test_pool().await;
        let user_id = create_test_user(&pool, "test_sn_list_user").await;
        let claims = test_claims(user_id, false);

        // 创建 3 条
        for i in 0..3 {
            let req = CreateShortNoteRequest {
                content: Some(format!("note_{}", i)),
                view_name: None,
            };
            create_short_note(Extension(pool.clone()), claims.clone(), Json(req))
                .await
                .unwrap();
        }

        // 列表
        let query = ListQuery { page: 1, page_size: 10 };
        let result = list_short_notes(Extension(pool.clone()), claims, Query(query)).await;
        assert!(result.is_ok());
        let notes = result.unwrap().data.unwrap();
        assert!(notes.len() >= 3);

        // 清理
        sqlx::query("DELETE FROM short_notes WHERE user_id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .ok();
        cleanup_test_user(&pool, user_id).await;
    }

    #[tokio::test]
    async fn test_update_short_note() {
        let pool = test_pool().await;
        let user_id = create_test_user(&pool, "test_sn_update_user").await;
        let claims = test_claims(user_id, false);

        let req = CreateShortNoteRequest {
            content: Some("原始内容".to_string()),
            view_name: None,
        };
        let note = create_short_note(Extension(pool.clone()), claims.clone(), Json(req))
            .await
            .unwrap()
            .data
            .unwrap();

        let update = UpdateShortNoteRequest {
            content: Some("更新后内容".to_string()),
            view_name: Some("新视图".to_string()),
        };
        let result =
            update_short_note(Extension(pool.clone()), claims, Path(note.id), Json(update)).await;
        assert!(result.is_ok());
        let updated = result.unwrap().data.unwrap();
        assert_eq!(updated.content.as_deref(), Some("更新后内容"));
        assert_eq!(updated.view_name.as_deref(), Some("新视图"));

        // 清理
        sqlx::query("DELETE FROM short_notes WHERE user_id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .ok();
        cleanup_test_user(&pool, user_id).await;
    }

    #[tokio::test]
    async fn test_delete_short_note() {
        let pool = test_pool().await;
        let user_id = create_test_user(&pool, "test_sn_delete_user").await;
        let claims = test_claims(user_id, false);

        let req = CreateShortNoteRequest {
            content: Some("待删除".to_string()),
            view_name: None,
        };
        let note = create_short_note(Extension(pool.clone()), claims.clone(), Json(req))
            .await
            .unwrap()
            .data
            .unwrap();

        let result = delete_short_note(Extension(pool.clone()), claims.clone(), Path(note.id)).await;
        assert!(result.is_ok());

        // 再次获取应报 NotFound
        let result2 = get_short_note(Extension(pool.clone()), claims, Path(note.id)).await;
        assert!(result2.is_err());
        match result2.unwrap_err() {
            ApiError::NotFound { .. } => {}
            other => panic!("期望 NotFound, 得到: {:?}", other),
        }

        cleanup_test_user(&pool, user_id).await;
    }

    #[tokio::test]
    async fn test_get_short_note_not_found() {
        let pool = test_pool().await;
        let user_id = create_test_user(&pool, "test_sn_404_user").await;
        let claims = test_claims(user_id, false);
        let fake_id = Uuid::now_v7();

        let result = get_short_note(Extension(pool.clone()), claims, Path(fake_id)).await;
        assert!(result.is_err());

        cleanup_test_user(&pool, user_id).await;
    }
}
