// WASM stub: ApiClient 功能在 WASM 中由 common crate 的 reqwest 提供
// 注意：WASM 目标下不要提供固定的测试 token —— 一律未认证，
// 认证需由调用方通过 init_client/set_client_token 注入真实 token。
use common::api::client::ApiClient;
use std::cell::RefCell;

thread_local! {
    static TOKEN: RefCell<Option<String>> = const { RefCell::new(None) };
    static BASE_URL: RefCell<String> = const { RefCell::new(String::new()) };
}

pub fn init_client(token: Option<String>, _port: Option<u16>) -> Result<(), String> {
    TOKEN.with(|t| *t.borrow_mut() = token);
    Ok(())
}

pub fn set_client_base_url(base_url: &str) -> Result<(), String> {
    BASE_URL.with(|b| *b.borrow_mut() = base_url.to_string());
    Ok(())
}

pub fn set_client_token(token: &str) -> Result<(), String> {
    TOKEN.with(|t| *t.borrow_mut() = Some(token.to_string()));
    Ok(())
}

pub fn clear_client_token() -> Result<(), String> {
    TOKEN.with(|t| *t.borrow_mut() = None);
    Ok(())
}

pub fn get_client_base_url() -> Result<String, String> {
    Ok(BASE_URL.with(|b| b.borrow().clone()))
}

pub fn get_client_clone() -> Result<ApiClient, String> {
    let base_url = BASE_URL.with(|b| b.borrow().clone());
    let mut client = ApiClient::new(if base_url.is_empty() { "http://localhost:23001" } else { &base_url });
    if let Some(token) = TOKEN.with(|t| t.borrow().clone()) {
        client.set_token(&token);
    }
    Ok(client)
}
