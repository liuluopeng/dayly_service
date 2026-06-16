use serde::{Deserialize, Serialize};
use uuid::Uuid;

// 会话模型
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(sqlx::FromRow))]
pub struct OpenAiSession {
    pub id: Uuid,
    pub title: String,
    pub user_id: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Local>,
    pub updated_at: chrono::DateTime<chrono::Local>,
}

// Cite项模型
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CiteItem {
    pub url: String,
    pub title: String,
    pub snippet: String,
    #[serde(rename = "cite_index")]
    pub cite_index: i32,
    #[serde(rename = "published_at")]
    pub published_at: Option<String>,
    #[serde(rename = "site_name")]
    pub site_name: String,
    #[serde(rename = "site_icon")]
    pub site_icon: String,
    #[serde(rename = "query_indexes")]
    pub query_indexes: Vec<i32>,
}

// 消息模型
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(sqlx::FromRow))]
pub struct OpenAiMessage {
    pub id: Uuid,
    pub session_id: Uuid,
    pub role: String, // user, assistant, system
    pub content: String,
    pub think: Option<String>,
    pub cite: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Local>,
}

crate::impl_display!(self, OpenAiSession,
    "ID"   => self.id,
    "标题" => self.title
);

crate::impl_display!(self, CiteItem,
    "标题" => self.title,
    "来源" => self.site_name
);

crate::impl_display!(self, OpenAiMessage,
    "角色" => self.role,
    "内容" => if self.content.len() > 80 { format!("{}...", &self.content[..80]) } else { self.content.clone() }
);

impl OpenAiSession {
    /// 返回格式化的显示字符串
    pub fn display(&self) -> String {
        format!(
            "{} - {}",
            self.title,
            self.updated_at.format("%Y-%m-%d %H:%M")
        )
    }
}

impl OpenAiMessage {
    /// 返回格式化的显示字符串
    pub fn display(&self) -> String {
        format!(
            "[{}] {}",
            self.role,
            self.content
        )
    }
}
