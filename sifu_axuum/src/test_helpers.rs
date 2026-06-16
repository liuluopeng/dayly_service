#![cfg(test)]

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

use crate::middleware::Claims;

/// 连接到测试数据库
pub async fn test_pool() -> PgPool {
    let uri = "postgres://dayly_db:dayly_db@0.0.0.0:5432/dayly_db";
    PgPoolOptions::new()
        .max_connections(2)
        .connect(uri)
        .await
        .expect("测试数据库连接失败")
}

/// 创建测试用的 Claims
pub fn test_claims(user_id: Uuid, is_admin: bool) -> Claims {
    Claims {
        id: user_id.to_string(),
        jti: Uuid::now_v7().to_string(),
        exp: (chrono::Utc::now().timestamp() + 86400) as usize,
        is_admin,
        username: "test_user".to_string(),
    }
}

/// 创建测试用户，返回 user_id
pub async fn create_test_user(pool: &PgPool, username: &str) -> Uuid {
    let id = Uuid::now_v7();
    let row = sqlx::query(
        "INSERT INTO users (id, username, password, language, flutter_theme)
         VALUES ($1, $2, 'test', 'zh', 'dark')
         ON CONFLICT (username) DO UPDATE SET username = EXCLUDED.username
         RETURNING id",
    )
    .bind(id)
    .bind(username)
    .fetch_one(pool)
    .await
    .expect("创建测试用户失败");
    row.get::<Uuid, _>("id")
}

/// 清理测试用户
pub async fn cleanup_test_user(pool: &PgPool, user_id: Uuid) {
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(pool)
        .await
        .ok();
}
