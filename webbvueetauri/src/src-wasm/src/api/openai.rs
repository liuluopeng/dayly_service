use common::api::openai::{AddMessageRequest, ChatCompletionRequest, CreateSessionRequest};
use serde_wasm_bindgen::to_value;
use uuid::Uuid;
use wasm_bindgen::prelude::*;

use crate::api::init::get_api_client;
use crate::console_log;

#[wasm_bindgen]
pub async fn create_session(title: &str) -> Result<JsValue, JsValue> {
    let client = get_api_client(None);

    match common::api::openai::create_session(client, title).await {
        Ok(response) => {
            console_log!("创建会话成功！");
            match to_value(&response) {
                Ok(js_value) => Ok(js_value),
                Err(e) => Err(JsValue::from_str(&format!("序列化失败: {}", e))),
            }
        }
        Err(error) => {
            console_log!("创建会话失败: {}", error);
            Err(JsValue::from_str(&format!("{}", error)))
        }
    }
}

#[wasm_bindgen]
pub async fn list_sessions() -> Result<JsValue, JsValue> {
    let client = get_api_client(None);

    match common::api::openai::list_sessions(client).await {
        Ok(response) => {
            console_log!("获取会话列表成功！");
            match to_value(&response) {
                Ok(js_value) => Ok(js_value),
                Err(e) => Err(JsValue::from_str(&format!("序列化失败: {}", e))),
            }
        }
        Err(error) => {
            console_log!("获取会话列表失败: {}", error);
            Err(JsValue::from_str(&format!("{}", error)))
        }
    }
}

#[wasm_bindgen]
pub async fn get_session(session_id: &str) -> Result<JsValue, JsValue> {
    let client = get_api_client(None);
    let session_id = Uuid::parse_str(session_id).map_err(|e| JsValue::from_str(&e.to_string()))?;

    match common::api::openai::get_session(client, &session_id).await {
        Ok(response) => {
            console_log!("获取会话详情成功！");
            match to_value(&response) {
                Ok(js_value) => Ok(js_value),
                Err(e) => Err(JsValue::from_str(&format!("序列化失败: {}", e))),
            }
        }
        Err(error) => {
            console_log!("获取会话详情失败: {}", error);
            Err(JsValue::from_str(&format!("{}", error)))
        }
    }
}

#[wasm_bindgen]
pub async fn delete_session(session_id: &str) -> Result<JsValue, JsValue> {
    let client = get_api_client(None);
    let session_id = Uuid::parse_str(session_id).map_err(|e| JsValue::from_str(&e.to_string()))?;

    match common::api::openai::delete_session(client, &session_id).await {
        Ok(response) => {
            console_log!("删除会话成功！");
            match to_value(&response) {
                Ok(js_value) => Ok(js_value),
                Err(e) => Err(JsValue::from_str(&format!("序列化失败: {}", e))),
            }
        }
        Err(error) => {
            console_log!("删除会话失败: {}", error);
            Err(JsValue::from_str(&format!("{}", error)))
        }
    }
}

#[wasm_bindgen]
pub async fn add_message(
    session_id: &str,
    role: &str,
    content: &str,
    think: Option<String>,
    cite: Option<String>,
) -> Result<JsValue, JsValue> {
    let client = get_api_client(None);
    let session_id = Uuid::parse_str(session_id).map_err(|e| JsValue::from_str(&e.to_string()))?;

    // 解析cite JSON字符串
    let cite_value = match cite {
        Some(cite_str) => {
            if cite_str.is_empty() {
                None
            } else {
                match serde_json::from_str(&cite_str) {
                    Ok(val) => Some(val),
                    Err(_) => None,
                }
            }
        }
        None => None,
    };

    match common::api::openai::add_message(
        client,
        &session_id,
        role,
        content,
        think.as_deref(),
        cite_value,
    )
    .await
    {
        Ok(response) => {
            console_log!("添加消息成功！");
            match to_value(&response) {
                Ok(js_value) => Ok(js_value),
                Err(e) => Err(JsValue::from_str(&format!("序列化失败: {}", e))),
            }
        }
        Err(error) => {
            console_log!("添加消息失败: {}", error);
            Err(JsValue::from_str(&format!("{}", error)))
        }
    }
}

#[wasm_bindgen]
pub async fn get_session_messages(session_id: &str) -> Result<JsValue, JsValue> {
    let client = get_api_client(None);
    let session_id = Uuid::parse_str(session_id).map_err(|e| JsValue::from_str(&e.to_string()))?;

    match common::api::openai::get_session_messages(client, &session_id).await {
        Ok(response) => {
            console_log!("获取消息列表成功！");
            match to_value(&response) {
                Ok(js_value) => Ok(js_value),
                Err(e) => Err(JsValue::from_str(&format!("序列化失败: {}", e))),
            }
        }
        Err(error) => {
            console_log!("获取消息列表失败: {}", error);
            Err(JsValue::from_str(&format!("{}", error)))
        }
    }
}

#[wasm_bindgen]
pub async fn chat_completion(request: &str) -> Result<JsValue, JsValue> {
    let client = get_api_client(None);
    let req: ChatCompletionRequest =
        serde_json::from_str(request).map_err(|e| JsValue::from_str(&e.to_string()))?;

    match common::api::openai::chat_completion(client, &req).await {
        Ok(response) => {
            console_log!("AI 对话成功！");
            match to_value(&response) {
                Ok(js_value) => Ok(js_value),
                Err(e) => Err(JsValue::from_str(&format!("序列化失败: {}", e))),
            }
        }
        Err(error) => {
            console_log!("AI 对话失败: {}", error);
            Err(JsValue::from_str(&format!("{}", error)))
        }
    }
}

#[wasm_bindgen]
pub async fn chat_completion_stream(request: &str) -> Result<JsValue, JsValue> {
    let client = get_api_client(None);
    let req: ChatCompletionRequest =
        serde_json::from_str(request).map_err(|e| JsValue::from_str(&e.to_string()))?;

    // 这里直接返回成功，因为流式传输需要在前端使用fetch API直接调用
    // WASM不适合处理流式响应，所以让前端直接调用流式API
    match to_value(&serde_json::json!({
        "msg": "ok",
        "data": null
    })) {
        Ok(js_value) => Ok(js_value),
        Err(e) => Err(JsValue::from_str(&format!("序列化失败: {}", e))),
    }
}
