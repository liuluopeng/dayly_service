use common::api::dict;
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;

use crate::{api::init::get_api_client, console_log};

#[wasm_bindgen]
pub async fn search_xiandaihanyu(word: &str) -> Result<JsValue, JsValue> {
    console_log!("搜索词语: {:?}", word);

    let client = get_api_client(None);

    match dict::search_xiandaihanyu(client, word).await {
        Ok(response) => {
            println!("请求成功！");
            println!("响应消息: {}", response.msg);
            console_log!("响应消息: {}", response.msg);
            if let Some(data) = response.data {
                println!("响应数据: {}", data);
                console_log!("响应数据: {}", data);

                // 将数据转换为JavaScript对象
                match to_value(&data) {
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
            // 将错误转换为JsValue
            Err(JsValue::from_str(&format!("{}", error)))
        }
    }
}

#[wasm_bindgen]
pub async fn search_collins(word: &str) -> Result<JsValue, JsValue> {
    console_log!("搜索词语: {:?}", word);

    let client = get_api_client(None);

    match dict::search_collins(client, word).await {
        Ok(response) => {
            println!("请求成功！");
            println!("响应消息: {}", response.msg);
            console_log!("响应消息: {}", response.msg);
            if let Some(data) = response.data {
                println!("响应数据: {}", data);
                console_log!("响应数据: {}", data);

                // 将数据转换为JavaScript对象
                match to_value(&data) {
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
            // 将错误转换为JsValue
            Err(JsValue::from_str(&format!("{}", error)))
        }
    }
}

#[wasm_bindgen]
pub async fn search_ldoce(word: &str) -> Result<JsValue, JsValue> {
    console_log!("搜索词语: {:?}", word);

    let client = get_api_client(None);

    match dict::search_ldoce(client, word).await {
        Ok(response) => {
            println!("请求成功！");
            println!("响应消息: {}", response.msg);
            console_log!("响应消息: {}", response.msg);
            if let Some(data) = response.data {
                println!("响应数据: {}", data);
                console_log!("响应数据: {}", data);

                // 将数据转换为JavaScript对象
                match to_value(&data) {
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
            // 将错误转换为JsValue
            Err(JsValue::from_str(&format!("{}", error)))
        }
    }
}

#[wasm_bindgen]
pub async fn chinese_dict(word: String) -> Result<JsValue, JsValue> {
    // 为了兼容旧的调用方式，chinese_dict函数调用search_xiandaihanyu函数
    search_xiandaihanyu(&word).await
}

#[wasm_bindgen]
pub async fn get_recent_history(limit: i64) -> Result<JsValue, JsValue> {
    console_log!("获取最近搜索历史，限制: {:?}", limit);

    let client = get_api_client(None);

    match dict::get_recent_history(client, limit).await {
        Ok(response) => {
            println!("请求成功！");
            println!("响应消息: {}", response.msg);
            console_log!("响应消息: {}", response.msg);

            if let Some(data) = &response.data {
                for item in data {
                    console_log!("响应数据: {:?}", item);
                }
            } 
            // 将整个响应转换为JavaScript对象
            match to_value(&response) {
                Ok(js_value) => Ok(js_value),
                Err(e) => Err(JsValue::from_str(&format!("序列化失败: {}", e))),
            }
        }
        Err(error) => {
            println!("请求失败: {}", error);
            console_log!("请求失败: {}", error);
            // 将错误转换为JsValue
            Err(JsValue::from_str(&format!("{}", error)))
        }
    }
}

#[wasm_bindgen]
pub async fn get_top_words() -> Result<JsValue, JsValue> {
    console_log!("获取热门词语");

    let client = get_api_client(None);

    match dict::get_top_words(client).await {
        Ok(response) => {
            println!("请求成功！");
            println!("响应消息: {}", response.msg);
            console_log!("响应消息: {}", response.msg);

            // 将整个响应转换为JavaScript对象
            match to_value(&response) {
                Ok(js_value) => Ok(js_value),
                Err(e) => Err(JsValue::from_str(&format!("序列化失败: {}", e))),
            }
        }
        Err(error) => {
            println!("请求失败: {}", error);
            console_log!("请求失败: {}", error);
            // 将错误转换为JsValue
            Err(JsValue::from_str(&format!("{}", error)))
        }
    }
}
