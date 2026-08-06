use js_sys::Array;
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;

use web_sys::console;

use common::front_can_do::zici::{NEW_CHARS_DATA, NEW_WORDS_DATA};

#[wasm_bindgen]
pub fn my_console_log(s: &str) {
    console::log_1(&JsValue::from_str(s));
}

#[wasm_bindgen]
pub fn get_new_chars(grade: usize, term: usize) -> JsValue {
    if !(1..=6).contains(&grade) || !(1..=2).contains(&term) {
        return Array::new().into();
    }
    let index = (grade - 1) * 2 + (term - 1);
    if index >= NEW_CHARS_DATA.len() {
        return Array::new().into();
    }
    let chars_str = NEW_CHARS_DATA[index];
    let array = Array::new();
    for c in chars_str.chars() {
        array.push(&JsValue::from_str(&c.to_string()));
    }
    array.into()
}

#[wasm_bindgen]
pub fn get_new_words() -> JsValue {
    let array = js_sys::Array::new();
    for word in NEW_WORDS_DATA.iter() {
        array.push(&JsValue::from_str(word));
    }
    array.into()
}

// 词频数据 — 由 build.rs 从 word_frequency_list.json 生成
include!(concat!(env!("OUT_DIR"), "/words_data.rs"));

/// 零拷贝：返回 [ptr, len] 指向紧凑结构体数组
#[wasm_bindgen]
pub fn get_words_data() -> JsValue {
    let ptr = WORDS_BLOB.as_ptr() as usize;
    let len = WORDS_BLOB.len();
    to_value(&(ptr as u32, len as u32)).unwrap()
}

#[wasm_bindgen]
pub fn get_word_count() -> usize {
    WORD_COUNT
}
#[wasm_bindgen]
pub fn get_entry_size() -> usize {
    ENTRY_SIZE
}

/// 按索引获取解释文本：返回 [ptr, len]
#[wasm_bindgen]
pub fn get_explanation_for_word(index: usize) -> JsValue {
    if index >= WORD_COUNT {
        return to_value(&(0u32, 0u32)).unwrap();
    }
    let idx_off = index * 8;
    let offset =
        u32::from_le_bytes(EXPLANATION_INDEX[idx_off..idx_off + 4].try_into().unwrap()) as usize;
    let length = u32::from_le_bytes(
        EXPLANATION_INDEX[idx_off + 4..idx_off + 8]
            .try_into()
            .unwrap(),
    ) as usize;
    let ptr = EXPLANATION_BLOB[offset..offset + length].as_ptr() as usize;
    to_value(&(ptr as u32, length as u32)).unwrap()
}

#[wasm_bindgen]
pub fn get_direction(x: i32, y: i32) -> String {
    match (x.cmp(&0), y.cmp(&0)) {
        (std::cmp::Ordering::Greater, std::cmp::Ordering::Equal) => "东".to_string(),
        (std::cmp::Ordering::Less, std::cmp::Ordering::Equal) => "西".to_string(),
        (std::cmp::Ordering::Equal, std::cmp::Ordering::Less) => "北".to_string(),
        (std::cmp::Ordering::Equal, std::cmp::Ordering::Greater) => "南".to_string(),
        (std::cmp::Ordering::Greater, std::cmp::Ordering::Less) => "东北".to_string(),
        (std::cmp::Ordering::Greater, std::cmp::Ordering::Greater) => "东南".to_string(),
        (std::cmp::Ordering::Less, std::cmp::Ordering::Less) => "西北".to_string(),
        (std::cmp::Ordering::Less, std::cmp::Ordering::Greater) => "西南".to_string(),
        (std::cmp::Ordering::Equal, std::cmp::Ordering::Equal) => "原点".to_string(),
    }
}
