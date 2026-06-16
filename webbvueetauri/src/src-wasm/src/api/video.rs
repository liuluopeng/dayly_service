use common::api::{
    client::ApiClient,
    videos::{list_videos, scan_videos},
};
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;

use crate::{api::init::get_api_client, console_log};

#[wasm_bindgen]
pub async fn scan_videos_wasm() -> Result<JsValue, JsValue> {
    let client = get_api_client(None);

    match scan_videos(client).await {
        Ok(response) => {
            console_log!("扫描视频成功！");
            match to_value(&response.data) {
                Ok(js_value) => Ok(js_value),
                Err(e) => Err(JsValue::from_str(&format!("序列化失败: {}", e))),
            }
        }
        Err(error) => {
            console_log!("扫描视频失败: {}", error);
            Err(JsValue::from_str(&format!("{}", error)))
        }
    }
}

#[wasm_bindgen]
pub async fn list_videos_wasm(
    folder: Option<String>,
    page: Option<u32>,
    page_size: Option<u32>,
) -> Result<JsValue, JsValue> {
    let client = get_api_client(None);

    match list_videos(client, folder.as_deref(), page, page_size).await {
        Ok(response) => {
            console_log!("获取视频列表成功！");
            match to_value(&response.data) {
                Ok(js_value) => Ok(js_value),
                Err(e) => Err(JsValue::from_str(&format!("序列化失败: {}", e))),
            }
        }
        Err(error) => {
            console_log!("获取视频列表失败: {}", error);
            Err(JsValue::from_str(&format!("{}", error)))
        }
    }
}
