use serde::{Deserialize, Serialize};
use uuid::Uuid;

// 仅在非 WebAssembly 环境中导入 sqlx
#[cfg(not(target_arch = "wasm32"))]
use sqlx::FromRow;

// 为所有环境定义结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(FromRow))]
pub struct AdminUser {
    /// 用户 ID
    pub id: Uuid,
    /// 用户名
    pub username: String,
    /// 密码哈希
    pub password_hash: String,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Local>,
    /// 更新时间
    pub updated_at: chrono::DateTime<chrono::Local>,
}

crate::impl_display!(self, AdminUser,
    "ID"     => self.id,
    "用户名" => self.username
);
