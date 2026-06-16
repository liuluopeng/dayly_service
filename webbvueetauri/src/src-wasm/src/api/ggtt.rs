use common::api::{client::ApiClient, ggtt::search_ggtt_code};
use my_type::dto::SearchRequest;
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;

use crate::{api::init::get_api_client, console_log};

#[wasm_bindgen]
pub async fn search_ggtt(code: &str) -> Result<JsValue, JsValue> {
    let client = get_api_client(None);
    let url = format!("{}/api/ggtt/search_ggtt", client.base_url);

    console_log!("url: {:?}", url);

    let req = SearchRequest {
        search: code.to_string(),
    };

    match search_ggtt_code(client, req).await {
        Ok(response) => {
            println!("请求成功！");
            println!("响应消息: {}", response.msg);
            console_log!("响应消息: {}", response.msg);
            if let Some(ggtt_code) = response.data {
                println!("响应数据: {}", ggtt_code);
                console_log!("响应数据: {}", ggtt_code);

                // 将GgttCode转换为JavaScript对象
                match to_value(&ggtt_code) {
                    Ok(js_value) => Ok(js_value),
                    Err(e) => Err(JsValue::from_str(&format!("序列化失败: {}", e))),
                }
            } else {
                Err(JsValue::from_str("无数据返回"))
            }
        }
        Err(error) => {
            println!("请求失败: {}", error);
            console_log!("请求失败: {}", error);
            // 将ApiError转换为JsValue
            Err(JsValue::from_str(&format!("{}", error)))
        }
    }
}
