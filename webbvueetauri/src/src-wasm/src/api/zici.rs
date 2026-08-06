//! zici 字词学习——数据在服务端 sqlite（与 wasm-demo/Flutter 同一 API 源）
//! 生字/生词/词频全部通过 API 查询，不再内嵌数据

use wasm_bindgen::prelude::*;

use crate::api::helpers::data_to_js;
use crate::api::init::get_api_client;

#[wasm_bindgen]
pub fn my_console_log(s: &str) {
    web_sys::console::log_1(&JsValue::from_str(s));
}

/// 生字表：按年级/学期（grade 1-6，term 1-2）——返回字符串数组
#[wasm_bindgen]
pub async fn get_new_chars(grade: usize, term: usize) -> Result<JsValue, JsValue> {
    let client = get_api_client(None);
    let resp = common::api::zici::zici_chars(&client, grade as u32, term as u32)
        .await
        .map_err(|e| JsValue::from_str(&format!("{}", e)))?;
    let value = serde_json::to_value(resp.data.unwrap_or_default())
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    data_to_js(value)
}

/// 生词表：返回前 500 个（搜索留空为全部）
#[wasm_bindgen]
pub async fn get_new_words() -> Result<JsValue, JsValue> {
    let client = get_api_client(None);
    let resp = common::api::zici::zici_words(&client, "", 1, 500)
        .await
        .map_err(|e| JsValue::from_str(&format!("{}", e)))?;
    let data = resp
        .data
        .and_then(|v| v["data"].as_array().cloned())
        .unwrap_or_default();
    let value = serde_json::to_value(data).map_err(|e| JsValue::from_str(&e.to_string()))?;
    data_to_js(value)
}

/// 词频搜索（含释义）——返回 [{word, pinyin, frequency, explanation}]
#[wasm_bindgen]
pub async fn get_word_frequency_api(search: String, limit: usize) -> Result<JsValue, JsValue> {
    let client = get_api_client(None);
    let resp = common::api::zici::zici_word_frequency(&client, &search, limit as u32)
        .await
        .map_err(|e| JsValue::from_str(&format!("{}", e)))?;
    let value = serde_json::to_value(resp.data.unwrap_or_default())
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    data_to_js(value)
}

#[wasm_bindgen]
pub fn get_direction(x: i32, y: i32) -> String {
    if x.abs() > y.abs() {
        if x > 0 {
            "right".into()
        } else {
            "left".into()
        }
    } else if y > 0 {
        "down".into()
    } else {
        "up".into()
    }
}
