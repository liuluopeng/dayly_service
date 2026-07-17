use common::api::clipboard::get_clipboard_history;
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;

use crate::{api::init::get_api_client, console_log};

#[wasm_bindgen]
pub async fn get_clipboard_history_wasm(
    count: Option<usize>,
    type_filter: Option<String>,
    search: Option<String>,
) -> Result<JsValue, JsValue> {
    let client = get_api_client(None);

    match get_clipboard_history(client, count, type_filter.as_deref(), search.as_deref()).await {
        Ok(entries) => {
            console_log!("获取剪贴板历史成功！");
            match to_value(&entries) {
                Ok(js_value) => Ok(js_value),
                Err(e) => Err(JsValue::from_str(&format!("序列化失败: {}", e))),
            }
        }
        Err(error) => {
            console_log!("获取剪贴板历史失败: {}", error);
            Err(JsValue::from_str(&format!("{}", error)))
        }
    }
}
