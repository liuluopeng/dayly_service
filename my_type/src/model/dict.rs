use chrono::Local;
use html2text;
use serde::{Deserialize, Serialize};

// 仅在非 WebAssembly 环境中导入 sqlx
#[cfg(not(target_arch = "wasm32"))]
use sqlx::FromRow;

// 为所有环境定义结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(FromRow))]
pub struct DictWord {
    pub word: String,
    pub explanation: String,
}

crate::impl_display!(self, DictWord,
    "词语" => self.word,
    "解释" => html2text::from_read(self.explanation.as_bytes(), 80).unwrap_or_else(|_| self.explanation.clone())
);

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(FromRow))]
pub struct ModernChineseWord {
    pub word: String,
    pub explanation: String,
}

crate::impl_display!(self, ModernChineseWord,
    "词语" => self.word,
    "解释" => html2text::from_read(self.explanation.as_bytes(), 80).unwrap_or_else(|_| self.explanation.clone())
);

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(FromRow))]
pub struct DictResource {
    pub resource_path: String,
    pub resource_data: Vec<u8>,
}

crate::impl_display!(self, DictResource,
    "路径"     => self.resource_path,
    "数据大小" => format!("{} bytes", self.resource_data.len())
);

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(FromRow))]
pub struct WordHistory {
    pub id: uuid::Uuid,
    pub word: String,
    pub time: chrono::DateTime<Local>,
    pub created_at: chrono::DateTime<Local>,
}

crate::impl_display!(self, WordHistory,
    "词语" => self.word,
    "时间" => self.time
);

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(FromRow))]
pub struct Word {
    pub id: uuid::Uuid,
    pub word: String,
    pub has_searched_times: i32,
}

crate::impl_display!(self, Word,
    "词语"   => self.word,
    "搜索次数" => self.has_searched_times
);
