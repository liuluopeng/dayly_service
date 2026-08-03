use common::api::client::ApiClient;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

// 全局 API 客户端实例（单线程 WASM，用 thread_local 避免 static mut 的
// 别名/use-after-free 问题）。所有调用方拿到的是客户端克隆，
// 不再跨 .await 持有 &'static mut。
thread_local! {
    static API_CLIENT: RefCell<Option<ApiClient>> = const { RefCell::new(None) };
    static CURRENT_TOKEN: RefCell<Option<String>> = const { RefCell::new(None) };
}

fn current_token() -> Option<String> {
    CURRENT_TOKEN.with(|t| t.borrow().clone())
}

// 初始化 API 客户端（URL/端口变化时重建；否则复用）
pub fn init_api_client(token: Option<&str>, api_url: Option<&str>, api_port: Option<&str>) {
    let api_url = api_url.unwrap_or(""); // 空串 = 相对路径（同源）

    // 如果传入了 token，保存到全局变量
    if let Some(token) = token {
        CURRENT_TOKEN.with(|t| *t.borrow_mut() = Some(token.to_string()));
    }

    // 仅在 base URL 变化时重建客户端，避免 drop 掉正在使用的客户端
    let url_changed = API_CLIENT.with(|cell| {
        let borrow = cell.borrow();
        match borrow.as_ref() {
            Some(c) => c.base_url() != api_url,
            None => true,
        }
    });
    if url_changed {
        API_CLIENT.with(|cell| *cell.borrow_mut() = Some(ApiClient::new(api_url)));
    }

    // 如果有保存的 token，设置到客户端
    if let Some(token) = current_token() {
        API_CLIENT.with(|cell| {
            if let Some(client) = cell.borrow_mut().as_mut() {
                client.set_token(&token);
            }
        });
    }

    // 如果指定了端口，设置端口
    if let Some(port) = api_port {
        if let Ok(port_num) = port.parse::<u16>() {
            API_CLIENT.with(|cell| {
                if let Some(client) = cell.borrow_mut().as_mut() {
                    client.set_port(port_num);
                }
            });
        }
    }
}

// 获取 API 客户端（克隆，安全跨 await）
pub fn get_api_client(token: Option<&str>) -> ApiClient {
    if let Some(token) = token {
        CURRENT_TOKEN.with(|t| *t.borrow_mut() = Some(token.to_string()));
    }
    let token = current_token();

    let initialized = API_CLIENT.with(|cell| cell.borrow().is_some());
    if !initialized {
        init_api_client(None, Some(""), None);
    }

    let mut cloned =
        API_CLIENT.with(|cell| cell.borrow().as_ref().expect("客户端已初始化").clone());
    if let Some(ref t) = token {
        cloned.set_token(t);
    }
    cloned
}

// 清除保存的 token（登出时调用）
pub fn clear_api_token() {
    CURRENT_TOKEN.with(|t| *t.borrow_mut() = None);
    API_CLIENT.with(|cell| {
        if let Some(client) = cell.borrow_mut().as_mut() {
            client.clear_token();
        }
    });
}

// 初始化 API 客户端（WASM 绑定）
#[wasm_bindgen]
pub fn init_api(token: Option<String>, api_url: Option<String>, port: Option<String>) {
    init_api_client(token.as_deref(), api_url.as_deref(), port.as_deref());
}

// 获取 API 基础 URL（WASM 绑定）
#[wasm_bindgen]
pub fn get_base_url_wasm() -> String {
    get_api_client(None).base_url().to_string()
}

// 清除 token（WASM 绑定，登出时调用）
#[wasm_bindgen]
pub fn clear_api_token_wasm() {
    clear_api_token();
}

// 设置API客户端端口（WASM绑定）
#[wasm_bindgen]
pub fn set_api_port(port: &str) {
    API_CLIENT.with(|cell| {
        let mut borrow = cell.borrow_mut();
        if let Some(client) = borrow.as_mut() {
            if let Ok(port_num) = port.parse::<u16>() {
                client.set_port(port_num);
            }
        }
    });
    if API_CLIENT.with(|cell| cell.borrow().is_none()) {
        // 如果客户端未初始化，先初始化再设置端口
        init_api_client(None, Some(""), Some(port));
    }
}
