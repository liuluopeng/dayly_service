use crate::middleware::Claims;
use axum::extract::{Extension, Path as AxumPath, Query};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use common::api::base::{ApiError, ApiResponse, ApiResult};
use my_type::model::user_directories::UserDirectory;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ListUserDirsQuery {
    pub username: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddUserDirRequest {
    pub path: String,
    pub label: Option<String>,
    pub allow_list: Option<Vec<String>>,
}

// 管理员：列出目录（可按用户名过滤）
pub async fn list_user_directories(
    claims: Claims,
    Query(query): Query<ListUserDirsQuery>,
    Extension(pg_pool): Extension<PgPool>,
) -> ApiResult<ApiResponse<Vec<UserDirectory>>> {
    if !claims.is_admin {
        return Err(ApiError::forbidden(ApiError::ADMIN_REQUIRED, "需要管理员权限"));
    }

    let dirs = if let Some(ref username) = query.username {
        sqlx::query_as::<_, UserDirectory>(
            "SELECT id, path, label, allow_list, created_at FROM user_directories WHERE $1 = ANY(allow_list) ORDER BY created_at",
        )
        .bind(username)
        .fetch_all(&pg_pool)
        .await
        .map_err(|e| ApiError::Internal(format!("查询目录失败: {}", e)))?
    } else {
        sqlx::query_as::<_, UserDirectory>(
            "SELECT id, path, label, allow_list, created_at FROM user_directories ORDER BY created_at",
        )
        .fetch_all(&pg_pool)
        .await
        .map_err(|e| ApiError::Internal(format!("查询目录失败: {}", e)))?
    };

    Ok(ApiResponse::ok(dirs))
}

// 管理员：添加目录
pub async fn add_user_directory(
    claims: Claims,
    Extension(pg_pool): Extension<PgPool>,
    Json(body): Json<AddUserDirRequest>,
) -> ApiResult<ApiResponse<UserDirectory>> {
    if !claims.is_admin {
        return Err(ApiError::forbidden(ApiError::ADMIN_REQUIRED, "需要管理员权限"));
    }

    if body.path.is_empty() {
        return Err(ApiError::bad_request(ApiError::EMPTY_PATH, "路径不能为空"));
    }

    let allow_list = body.allow_list.unwrap_or_default();

    let dir = sqlx::query_as::<_, UserDirectory>(
        "INSERT INTO user_directories (id, path, label, allow_list) VALUES ($1, $2, $3, $4)
         ON CONFLICT (path) DO UPDATE SET label = EXCLUDED.label, allow_list = EXCLUDED.allow_list
         RETURNING id, path, label, allow_list, created_at",
    )
    .bind(Uuid::now_v7())
    .bind(&body.path)
    .bind(&body.label.unwrap_or_default())
    .bind(&allow_list)
    .fetch_one(&pg_pool)
    .await
    .map_err(|e| ApiError::Internal(format!("添加目录失败: {}", e)))?;

    Ok(ApiResponse::ok(dir))
}

// 管理员：删除目录（级联删除 media_paths）
pub async fn delete_user_directory(
    claims: Claims,
    Extension(pg_pool): Extension<PgPool>,
    AxumPath(dir_id): AxumPath<Uuid>,
) -> ApiResult<ApiResponse<()>> {
    if !claims.is_admin {
        return Err(ApiError::forbidden(ApiError::ADMIN_REQUIRED, "需要管理员权限"));
    }

    let result = sqlx::query(
        "DELETE FROM user_directories WHERE id = $1",
    )
    .bind(dir_id)
    .execute(&pg_pool)
    .await
    .map_err(|e| ApiError::Internal(format!("删除目录失败: {}", e)))?;

    if result.rows_affected() == 0 {
        return Err(ApiError::not_found(ApiError::DIR_NOT_FOUND, "目录不存在"));
    }

    Ok(ApiResponse::ok(()))
}

pub fn admin_user_dir_routes() -> Router {
    Router::new()
        .route("/", get(list_user_directories).post(add_user_directory))
        .route("/{id}", delete(delete_user_directory))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;
    use axum::Json;
    use axum::extract::{Path as AxumPath, Query};

    #[tokio::test]
    async fn test_admin_add_list_delete_directory() {
        let pool = test_pool().await;
        let admin_id = create_test_user(&pool, "test_dir_admin").await;
        let admin_claims = test_claims(admin_id, true); // is_admin = true

        // 添加目录
        let req = AddUserDirRequest {
            path: format!("/tmp/test_dir_{}", Uuid::now_v7()),
            label: Some("测试目录".to_string()),
            allow_list: Some(vec!["test_dir_admin".to_string()]),
        };
        let result = add_user_directory(admin_claims.clone(), Extension(pool.clone()), Json(req)).await;
        assert!(result.is_ok());
        let dir = result.unwrap().data.unwrap();
        assert_eq!(dir.label.as_deref(), Some("测试目录"));

        // 列出目录
        let query = ListUserDirsQuery { username: None };
        let result2 = list_user_directories(admin_claims.clone(), Query(query), Extension(pool.clone())).await;
        assert!(result2.is_ok());
        let dirs = result2.unwrap().data.unwrap();
        assert!(dirs.iter().any(|d| d.id == dir.id));

        // 删除目录
        let result3 = delete_user_directory(admin_claims, Extension(pool.clone()), AxumPath(dir.id)).await;
        assert!(result3.is_ok());

        cleanup_test_user(&pool, admin_id).await;
    }

    #[tokio::test]
    async fn test_non_admin_forbidden() {
        let pool = test_pool().await;
        let user_id = create_test_user(&pool, "test_dir_nonadmin").await;
        let claims = test_claims(user_id, false); // is_admin = false

        let query = ListUserDirsQuery { username: None };
        let result = list_user_directories(claims.clone(), Query(query), Extension(pool.clone())).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::Forbidden { .. } => {}
            other => panic!("期望 Forbidden, 得到: {:?}", other),
        }

        cleanup_test_user(&pool, user_id).await;
    }
}
