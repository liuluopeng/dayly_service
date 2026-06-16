use serde::{Deserialize, Serialize};
use uuid::Uuid;

// 为所有环境定义结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(sqlx::FromRow))]
pub struct ViewName {
    pub id: Uuid,
    pub view_name: Option<String>,
    pub created_at: chrono::DateTime<chrono::Local>,
}

crate::impl_display!(self, ViewName,
    "ID"   => self.id,
    "名称" => self.view_name.as_deref().unwrap_or("无")
);
