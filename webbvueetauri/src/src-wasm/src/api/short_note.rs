use common::api::{
    client::ApiClient,
    short_note::{
        create_short_note, delete_short_note, get_short_note, list_short_notes, update_short_note,
        CreateShortNoteRequest, UpdateShortNoteRequest,
    },
};
use serde_wasm_bindgen::to_value;
use uuid::Uuid;
use wasm_bindgen::prelude::*;

use crate::{api::init::get_api_client, console_log};

/// 获取短笔记列表
#[wasm_bindgen]
pub async fn list_short_notes_wasm(
    page: Option<u32>,
    page_size: Option<u32>,
) -> Result<JsValue, JsValue> {
    let client = get_api_client(None);

    match list_short_notes(client, page, page_size).await {
        Ok(response) => {
            console_log!("获取短笔记列表成功！");
            match to_value(&response) {
                Ok(js_value) => Ok(js_value),
                Err(e) => Err(JsValue::from_str(&format!("序列化失败: {}", e))),
            }
        }
        Err(error) => {
            console_log!("获取短笔记列表失败: {}", error);
            Err(JsValue::from_str(&format!("{}", error)))
        }
    }
}

/// 获取单个短笔记
#[wasm_bindgen]
pub async fn get_short_note_wasm(id: &str) -> Result<JsValue, JsValue> {
    let client = get_api_client(None);

    let uuid = match Uuid::parse_str(id) {
        Ok(u) => u,
        Err(e) => return Err(JsValue::from_str(&format!("无效的UUID: {}", e))),
    };

    match get_short_note(client, uuid).await {
        Ok(response) => {
            console_log!("获取短笔记成功！");
            match to_value(&response) {
                Ok(js_value) => Ok(js_value),
                Err(e) => Err(JsValue::from_str(&format!("序列化失败: {}", e))),
            }
        }
        Err(error) => {
            console_log!("获取短笔记失败: {}", error);
            Err(JsValue::from_str(&format!("{}", error)))
        }
    }
}

/// 创建短笔记
#[wasm_bindgen]
pub async fn create_short_note_wasm(
    content: Option<String>,
    view_name: Option<String>,
) -> Result<JsValue, JsValue> {
    let client = get_api_client(None);

    let req = CreateShortNoteRequest { content, view_name };

    match create_short_note(client, req).await {
        Ok(response) => {
            console_log!("创建短笔记成功！");
            match to_value(&response) {
                Ok(js_value) => Ok(js_value),
                Err(e) => Err(JsValue::from_str(&format!("序列化失败: {}", e))),
            }
        }
        Err(error) => {
            console_log!("创建短笔记失败: {}", error);
            Err(JsValue::from_str(&format!("{}", error)))
        }
    }
}

/// 更新短笔记
#[wasm_bindgen]
pub async fn update_short_note_wasm(
    id: &str,
    content: Option<String>,
    view_name: Option<String>,
) -> Result<JsValue, JsValue> {
    let client = get_api_client(None);

    let uuid = match Uuid::parse_str(id) {
        Ok(u) => u,
        Err(e) => return Err(JsValue::from_str(&format!("无效的UUID: {}", e))),
    };

    let req = UpdateShortNoteRequest { content, view_name };

    match update_short_note(client, uuid, req).await {
        Ok(response) => {
            console_log!("更新短笔记成功！");
            match to_value(&response) {
                Ok(js_value) => Ok(js_value),
                Err(e) => Err(JsValue::from_str(&format!("序列化失败: {}", e))),
            }
        }
        Err(error) => {
            console_log!("更新短笔记失败: {}", error);
            Err(JsValue::from_str(&format!("{}", error)))
        }
    }
}

/// 删除短笔记
#[wasm_bindgen]
pub async fn delete_short_note_wasm(id: &str) -> Result<JsValue, JsValue> {
    let client = get_api_client(None);

    let uuid = match Uuid::parse_str(id) {
        Ok(u) => u,
        Err(e) => return Err(JsValue::from_str(&format!("无效的UUID: {}", e))),
    };

    match delete_short_note(client, uuid).await {
        Ok(response) => {
            console_log!("删除短笔记成功！");
            match to_value(&response) {
                Ok(js_value) => Ok(js_value),
                Err(e) => Err(JsValue::from_str(&format!("序列化失败: {}", e))),
            }
        }
        Err(error) => {
            console_log!("删除短笔记失败: {}", error);
            Err(JsValue::from_str(&format!("{}", error)))
        }
    }
}
