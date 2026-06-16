use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(sqlx::FromRow))]
pub struct ChatMessage {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub content: String,
    pub created_at: chrono::DateTime<chrono::Local>,
}

crate::impl_display!(self, ChatMessage,
    "发送者" => self.sender_id,
    "内容" => if self.content.len() > 80 { format!("{}...", &self.content[..80]) } else { self.content.clone() }
);
