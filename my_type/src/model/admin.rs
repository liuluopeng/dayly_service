use serde::{Deserialize, Serialize};
use uuid::Uuid;

// 仅在非 WebAssembly 环境中导入 sqlx
#[cfg(not(target_arch = "wasm32"))]
use sqlx::FromRow;

// 为所有环境定义结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(FromRow))]
pub struct AdminUser {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub created_at: chrono::DateTime<chrono::Local>,
    pub updated_at: chrono::DateTime<chrono::Local>,
}

crate::impl_display!(self, AdminUser,
    "ID"     => self.id,
    "用户名" => self.username
);
