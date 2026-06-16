use wasm_bindgen::prelude::*;
use crate::api::init::get_api_client;
use super::helpers::{data_array_to_js, data_to_js};

#[wasm_bindgen]
pub async fn list_media_paths_wasm(media_type: Option<String>) -> Result<JsValue, JsValue> {
    let client = get_api_client(None);
    let result = common::api::media_paths::list_media_paths(&client, media_type.as_deref())
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    data_array_to_js(result)
}

#[wasm_bindgen]
pub async fn add_media_path_wasm(
    directory_id: &str,
    media_type: &str,
    path: &str,
    label: Option<String>,
) -> Result<JsValue, JsValue> {
    let client = get_api_client(None);
    let result = common::api::media_paths::add_media_path(
        &client,
        directory_id,
        media_type,
        path,
        label.as_deref(),
    )
    .await
    .map_err(|e| JsValue::from_str(&e.to_string()))?;
    data_to_js(result)
}

#[wasm_bindgen]
pub async fn delete_media_path_wasm(path_id: &str) -> Result<JsValue, JsValue> {
    let client = get_api_client(None);
    let result = common::api::media_paths::delete_media_path(&client, path_id)
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    data_to_js(result)
}
