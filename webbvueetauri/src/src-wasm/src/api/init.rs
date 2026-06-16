use common::api::client::ApiClient;
use wasm_bindgen::prelude::*;

// 全局API客户端实例
static mut API_CLIENT: Option<ApiClient> = None;
static mut CURRENT_TOKEN: Option<String> = None;

// 初始化API客户端
pub fn init_api_client(
    token: Option<&str>,
    api_url: Option<&str>,
    api_port: Option<&str>,
) -> &'static mut ApiClient {
    unsafe {
        // 从参数获取 API URL，如果没有则使用默认值
        let api_url = api_url.unwrap_or_else(|| panic!("API URL is required"));

        // 如果传入了token，保存到全局变量
        if let Some(token) = token {
            CURRENT_TOKEN = Some(token.to_string());
        }

        // 重新创建 API 客户端（支持端口切换）
        API_CLIENT = Some(ApiClient::new(api_url));

        // 如果有保存的 token，设置到客户端
        if let Some(ref token) = CURRENT_TOKEN {
            if let Some(client) = &mut API_CLIENT {
                client.set_token(token);
            }
        }

        // 如果指定了端口，设置端口
        if let Some(port) = api_port {
            if let Some(port_num) = port.parse::<u16>().ok() {
                if let Some(client) = &mut API_CLIENT {
                    client.set_port(port_num);
                }
            }
        }

        API_CLIENT.as_mut().unwrap()
    }
}

// 获取API客户端
pub fn get_api_client(token: Option<&str>) -> &'static mut ApiClient {
    unsafe {
        // 如果已经初始化了客户端，直接返回
        if let Some(ref mut client) = API_CLIENT {
            if let Some(token) = token {
                client.set_token(token);
            }
            return API_CLIENT.as_mut().unwrap();
        }
    }
    // 如果没有初始化，使用默认URL和端口初始化
    init_api_client(token, Some("http://localhost:23001"), None)
}

// 初始化 API 客户端（WASM 绑定）
#[wasm_bindgen]
pub fn init_api(token: Option<String>, api_url: Option<String>, port: Option<String>) {
    init_api_client(token.as_deref(), api_url.as_deref(), port.as_deref());
}

// 获取 API 基础 URL（WASM 绑定）
#[wasm_bindgen]
pub fn get_base_url_wasm() -> String {
    let client = get_api_client(None);
    client.base_url().to_string()
}

// 设置API客户端端口（WASM绑定）
#[wasm_bindgen]
pub fn set_api_port(port: &str) {
    unsafe {
        if let Some(client) = &mut API_CLIENT {
            if let Ok(port_num) = port.parse::<u16>() {
                client.set_port(port_num);
            }
        } else {
            // 如果客户端未初始化，先初始化再设置端口
            init_api_client(None, Some("http://localhost:23001"), Some(port));
        }
    }
}
