use serde::{Deserialize, Serialize};

// 仅在非 WebAssembly 环境中导入 sqlx
#[cfg(not(target_arch = "wasm32"))]
use sqlx::FromRow;

// 为所有环境定义结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(FromRow))]
pub struct SingleCharPinyin {
    pub id: i32,
    pub pinyin: String,
    pub ori: String,
    pub count: i32,
    pub pinyin_length: i32,
    pub first_letter: String,
}

crate::impl_display!(self, SingleCharPinyin,
    "原字" => self.ori,
    "拼音" => self.pinyin
);
