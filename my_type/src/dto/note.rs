use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(sqlx::FromRow))]
pub struct NoteSummary {
    pub id: Uuid,
    pub text: Option<String>,
    pub simple_text: Option<String>,
    pub filepath: Option<String>,
    pub filename: Option<String>,
    pub file_info: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Local>,
    pub updated_at: chrono::DateTime<chrono::Local>,
    pub sha256: Option<String>,
}

crate::impl_display!(self, NoteSummary,
    "ID"     => self.id,
    "文件名" => self.filename.as_deref().unwrap_or("无"),
    "内容"   => self.simple_text.as_ref().or(self.text.as_ref()).map(|t| {
        if t.len() > 50 { format!("{}...", &t[..50]) } else { t.clone() }
    }).unwrap_or_else(|| "无".to_string())
);
