// WASM stub: ApiClient 功能在 WASM 中由 common crate 的 reqwest 提供
use common::api::client::ApiClient;

static CLIENT: std::sync::OnceLock<ApiClient> = std::sync::OnceLock::new();

#[allow(unused_variables)]
pub fn init_client(token: Option<String>, port: Option<u16>) -> Result<(), String> {
    Ok(())
}

pub fn set_client_base_url(base_url: &str) -> Result<(), String> {
    Ok(())
}

pub fn set_client_token(token: &str) -> Result<(), String> {
    Ok(())
}

pub fn clear_client_token() -> Result<(), String> {
    Ok(())
}

pub fn get_client_base_url() -> Result<String, String> {
    Ok(String::new())
}

pub fn get_client_clone() -> Result<ApiClient, String> {
    Ok(ApiClient::tester_client())
}
