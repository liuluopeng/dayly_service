use common::api::{
    client::ApiClient,
    images::{get_image_folders, list_images, scan_images},
};
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;

use crate::{api::init::get_api_client, console_log};

#[wasm_bindgen]
pub async fn scan_images_wasm() -> Result<JsValue, JsValue> {
    let client = get_api_client(None);

    match scan_images(client).await {
        Ok(response) => {
            console_log!("扫描图片成功！");
            match to_value(&response.data) {
                Ok(js_value) => Ok(js_value),
                Err(e) => Err(JsValue::from_str(&format!("序列化失败: {}", e))),
            }
        }
        Err(error) => {
            console_log!("扫描图片失败: {}", error);
            Err(JsValue::from_str(&format!("{}", error)))
        }
    }
}

#[wasm_bindgen]
pub async fn get_image_folders_wasm() -> Result<JsValue, JsValue> {
    let client = get_api_client(None);

    match get_image_folders(client).await {
        Ok(response) => {
            console_log!("获取图片文件夹成功！");
            match to_value(&response.data) {
                Ok(js_value) => Ok(js_value),
                Err(e) => Err(JsValue::from_str(&format!("序列化失败: {}", e))),
            }
        }
        Err(error) => {
            console_log!("获取图片文件夹失败: {}", error);
            Err(JsValue::from_str(&format!("{}", error)))
        }
    }
}

#[wasm_bindgen]
pub async fn list_images_wasm(
    folder: Option<String>,
    page: Option<u32>,
    page_size: Option<u32>,
) -> Result<JsValue, JsValue> {
    let client = get_api_client(None);

    match list_images(client, folder.as_deref(), page, page_size).await {
        Ok(response) => {
            console_log!("获取图片列表成功！");
            match to_value(&response.data) {
                Ok(js_value) => Ok(js_value),
                Err(e) => Err(JsValue::from_str(&format!("序列化失败: {}", e))),
            }
        }
        Err(error) => {
            console_log!("获取图片列表失败: {}", error);
            Err(JsValue::from_str(&format!("{}", error)))
        }
    }
}
