use serde::{Deserialize, Serialize};
use serde_json::Value;

// 仅在非 WebAssembly 环境中导入 sqlx
#[cfg(not(target_arch = "wasm32"))]
use sqlx::FromRow;

// 为所有环境定义结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(FromRow))]
pub struct HaiyuDict {
    pub id: i32,
    pub word: String,
    pub pinyin_flat: String,
    pub frequency: i32,
    pub word_length: i32,
    pub first_char: String,
    pub pinyin_no_flat: Value,
}

crate::impl_display!(self, HaiyuDict,
    "词语" => self.word,
    "拼音" => self.pinyin_flat
);
