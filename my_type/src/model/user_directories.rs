use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(not(target_arch = "wasm32"))]
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(FromRow))]
pub struct UserDirectory {
    pub id: Uuid,
    pub path: String,
    pub label: Option<String>,
    pub allow_list: Option<Vec<String>>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

crate::impl_display!(self, UserDirectory,
    "ID"   => self.id,
    "路径" => self.path,
    "标签" => self.label.as_deref().unwrap_or("无")
);
