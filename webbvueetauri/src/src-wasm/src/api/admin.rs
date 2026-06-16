use wasm_bindgen::prelude::*;
use crate::api::init::get_api_client;
use super::helpers::{data_array_to_js, data_to_js};

#[wasm_bindgen]
pub async fn list_users_wasm() -> Result<JsValue, JsValue> {
    let client = get_api_client(None);
    let result = common::api::user_directories::list_users(&client)
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    data_array_to_js(result)
}

#[wasm_bindgen]
pub async fn list_user_directories_wasm(username: Option<String>) -> Result<JsValue, JsValue> {
    let client = get_api_client(None);
    let result = common::api::user_directories::list_user_directories(&client, username.as_deref())
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    data_array_to_js(result)
}

#[wasm_bindgen]
pub async fn add_user_directory_wasm(
    path: &str,
    label: Option<String>,
    allow_list: Option<String>,
) -> Result<JsValue, JsValue> {
    let client = get_api_client(None);
    let allow_list_vec = allow_list.map(|s| {
        s.split(',')
            .map(|item| item.trim().to_string())
            .filter(|item| !item.is_empty())
            .collect()
    });
    let result = common::api::user_directories::add_user_directory(
        &client,
        path,
        label.as_deref(),
        allow_list_vec,
    )
    .await
    .map_err(|e| JsValue::from_str(&e.to_string()))?;
    data_to_js(result)
}

#[wasm_bindgen]
pub async fn delete_user_directory_wasm(dir_id: &str) -> Result<JsValue, JsValue> {
    let client = get_api_client(None);
    let result = common::api::user_directories::delete_user_directory(&client, dir_id)
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    data_to_js(result)
}
