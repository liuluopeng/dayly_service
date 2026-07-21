use common::api::{
    client::ApiClient,
    user::{user_login, LoginRequest},
    ggtt::{search_ggtt_code, SearchRequest},
};
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;

static mut CLIENT: Option<ApiClient> = None;

fn get_client() -> &'static mut ApiClient {
    unsafe {
        if CLIENT.is_none() {
            CLIENT = Some(ApiClient::new("http://localhost:23001"));
        }
        CLIENT.as_mut().unwrap()
    }
}

#[wasm_bindgen]
pub fn init_api(port: Option<String>) {
    let c = get_client();
    if let Some(p) = port {
        if let Ok(n) = p.parse::<u16>() {
            c.set_port(n);
        }
    }
}

#[wasm_bindgen]
pub fn set_token(token: &str) {
    get_client().set_token(token);
}

#[wasm_bindgen]
pub async fn login(username: &str, password: &str) -> Result<JsValue, JsValue> {
    let client = get_client();
    match user_login(client, username, password).await {
        Ok(resp) => {
            if let Some(data) = resp.data {
                let obj = js_sys::Object::new();
                js_sys::Reflect::set(&obj, &"token".into(), &data.token.into()).ok();
                js_sys::Reflect::set(&obj, &"token_type".into(), &data.token_type.into()).ok();
                Ok(obj.into())
            } else {
                Err("登录失败: 无数据返回".into())
            }
        }
        Err(e) => Err(JsValue::from_str(&format!("{}", e))),
    }
}

#[wasm_bindgen]
pub async fn search_wubi(code: &str) -> Result<JsValue, JsValue> {
    let client = get_client();
    let req = SearchRequest { search: code.to_string() };
    match search_ggtt_code(client, req).await {
        Ok(resp) => {
            if let Some(data) = resp.data {
                to_value(&data).map_err(|e| JsValue::from_str(&format!("序列化失败: {}", e)))
            } else {
                Err("无数据返回".into())
            }
        }
        Err(e) => Err(JsValue::from_str(&format!("{}", e))),
    }
}
