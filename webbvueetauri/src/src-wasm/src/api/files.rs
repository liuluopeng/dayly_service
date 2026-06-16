use common::api::files::{list_files, get_file_info, build_file_url, generate_tree};
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;

use crate::{api::init::get_api_client, console_log};

#[wasm_bindgen]
pub async fn list_files_wasm(path: &str, page: Option<usize>, page_size: Option<usize>) -> Result<JsValue, JsValue> {
    let client = get_api_client(None);

    match list_files(&client, path, page, page_size).await {
        Ok(listing) => {
            console_log!("列出目录: {} ({} 项, 共 {} 项)", listing.path, listing.entries.len(), listing.total);
            to_value(&listing).map_err(|e| JsValue::from_str(&format!("序列化失败: {}", e)))
        }
        Err(error) => {
            console_log!("列出目录失败: {}", error);
            Err(JsValue::from_str(&format!("{}", error)))
        }
    }
}

#[wasm_bindgen]
pub async fn get_file_info_wasm(path: &str) -> Result<JsValue, JsValue> {
    let client = get_api_client(None);

    match get_file_info(&client, path).await {
        Ok(info) => {
            console_log!("文件信息: {} ({})", info.name, info.content_type);
            to_value(&info).map_err(|e| JsValue::from_str(&format!("序列化失败: {}", e)))
        }
        Err(error) => {
            console_log!("获取文件信息失败: {}", error);
            Err(JsValue::from_str(&format!("{}", error)))
        }
    }
}

#[wasm_bindgen]
pub fn get_file_url_wasm(path: &str) -> String {
    let client = get_api_client(None);
    build_file_url(&client, path)
}

#[wasm_bindgen]
pub async fn generate_tree_wasm(path: &str) -> Result<JsValue, JsValue> {
    let client = get_api_client(None);

    match generate_tree(&client, path).await {
        Ok(result) => {
            console_log!("生成目录树: {}", path);
            to_value(&result).map_err(|e| JsValue::from_str(&format!("序列化失败: {}", e)))
        }
        Err(error) => {
            console_log!("生成目录树失败: {}", error);
            Err(JsValue::from_str(&format!("{}", error)))
        }
    }
}
