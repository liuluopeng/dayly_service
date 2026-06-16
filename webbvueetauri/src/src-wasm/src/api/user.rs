use common::api::{
    client::ApiClient,
    user::{user_login, LoginRequest},
};
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;

use crate::{
    api::init::{get_api_client, init_api_client},
    console_log,
};

#[wasm_bindgen]
pub async fn login_wasm(
    username: &str,
    password: &str,
    token: Option<String>,
    port: Option<String>,
) -> Result<JsValue, JsValue> {
    let client = get_api_client(token.as_deref());

    match user_login(client, username, password).await {
        Ok(response) => {
            console_log!("登录成功！");
            if let Some(login_response) = response.data {
                console_log!("Token: {}", login_response.token);
                console_log!("Token Type: {}", login_response.token_type);

                match to_value(&login_response) {
                    Ok(js_value) => Ok(js_value),
                    Err(e) => Err(JsValue::from_str(&format!("序列化失败: {}", e))),
                }
            } else {
                Err(JsValue::from_str("登录失败: 无数据返回"))
            }
        }
        Err(error) => {
            console_log!("登录失败: {}", error);
            Err(JsValue::from_str(&format!("{}", error)))
        }
    }
}
