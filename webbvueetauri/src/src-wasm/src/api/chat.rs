use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;

use crate::api::init::get_api_client;
use crate::console_log;

#[wasm_bindgen]
pub async fn send_message_wasm(content: &str) -> Result<JsValue, JsValue> {
    let client = get_api_client(None);

    match common::api::chat::send_message(client, content).await {
        Ok(response) => {
            console_log!("发送消息成功！");
            match to_value(&response) {
                Ok(js_value) => Ok(js_value),
                Err(e) => Err(JsValue::from_str(&format!("序列化失败: {}", e))),
            }
        }
        Err(error) => {
            console_log!("发送消息失败: {}", error);
            Err(JsValue::from_str(&format!("{}", error)))
        }
    }
}

#[wasm_bindgen]
pub async fn get_messages_wasm(after: Option<String>) -> Result<JsValue, JsValue> {
    let client = get_api_client(None);

    match common::api::chat::get_messages(client, after.as_deref()).await {
        Ok(response) => {
            console_log!("获取消息成功！");
            match to_value(&response) {
                Ok(js_value) => Ok(js_value),
                Err(e) => Err(JsValue::from_str(&format!("序列化失败: {}", e))),
            }
        }
        Err(error) => {
            console_log!("获取消息失败: {}", error);
            Err(JsValue::from_str(&format!("{}", error)))
        }
    }
}

#[wasm_bindgen]
pub async fn get_recent_contacts_wasm() -> Result<JsValue, JsValue> {
    let client = get_api_client(None);

    match common::api::chat::get_recent_contacts(client).await {
        Ok(response) => match to_value(&response) {
            Ok(js_value) => Ok(js_value),
            Err(e) => Err(JsValue::from_str(&format!("序列化失败: {}", e))),
        },
        Err(error) => Err(JsValue::from_str(&format!("{}", error))),
    }
}

#[wasm_bindgen]
pub async fn get_contacts_wasm() -> Result<JsValue, JsValue> {
    let client = get_api_client(None);

    match common::api::chat::get_contacts(client).await {
        Ok(response) => match to_value(&response) {
            Ok(js_value) => Ok(js_value),
            Err(e) => Err(JsValue::from_str(&format!("序列化失败: {}", e))),
        },
        Err(error) => Err(JsValue::from_str(&format!("{}", error))),
    }
}
