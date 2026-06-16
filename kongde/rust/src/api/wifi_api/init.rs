use lazy_static::lazy_static;
use std::sync::Mutex;


// 公开导出 ApiClient 类型，以便生成的代码可以访问它
pub use common::api::client::ApiClient;

// 全局可变的API客户端实例
lazy_static! {
    static ref GLOBAL_CLIENT: Mutex<ApiClient> = Mutex::new(ApiClient::default());
}

/// 初始化API客户端
/// Flutter在启动时调用此函数来初始化一个可变的reqwest客户端
///
/// # 参数
/// - `token`: 可选的认证令牌
/// - `port`: 服务器端口号，如果为None则使用默认端口23001
pub fn init_client(token: Option<String>, port: Option<u16>) -> Result<(), String> {
    // 尝试获取锁并初始化客户端
    match GLOBAL_CLIENT.lock() {
        Ok(mut client) => {
            // 使用默认端口初始化客户端
            let port = port.unwrap_or(23001);
            client.set_port(port);

            if let Some(token) = token {
                client.set_token(&token);
            }
            Ok(())
        }
        Err(e) => Err(format!("Failed to initialize client: {}", e)),
    }
}

/// 获取全局API客户端实例的引用
/// 注意：返回的是一个函数，该函数接收一个闭包来操作客户端
pub fn with_client<F, R>(f: F) -> Result<R, String>
where
    F: FnOnce(&mut ApiClient) -> R,
{
    match GLOBAL_CLIENT.lock() {
        Ok(mut client) => Ok(f(&mut client)),
        Err(e) => Err(format!("Failed to get client: {}", e)),
    }
}

/// 设置API客户端的基础URL
pub fn set_client_base_url(base_url: &str) -> Result<(), String> {
    with_client(|client| {
        client.set_base_url(base_url);
    })
}

/// 设置API客户端的token
pub fn set_client_token(token: &str) -> Result<(), String> {
    with_client(|client| {
        client.set_token(token);
    })
}

/// 清除API客户端的token
pub fn clear_client_token() -> Result<(), String> {
    with_client(|client| {
        client.clear_token();
    })
}

/// 获取API客户端的基础URL
pub fn get_client_base_url() -> Result<String, String> {
    with_client(|client| client.base_url().to_string())
}

/// 获取全局API客户端的克隆
/// 这样就可以直接传递给像`get_top_words_for_dart(&client)`这样的函数
pub fn get_client_clone() -> Result<ApiClient, String> {
    with_client(|client| {
        // 创建一个新的客户端实例，复制当前的配置
        let mut new_client = ApiClient::new(client.base_url());
        if let Some(token) = client.token() {
            new_client.set_token(token);
        }
        new_client
    })
}

/// 直接执行HTTP请求的函数，类似于数据库的pool()函数
/// 这样API函数就可以直接使用，不需要传递client参数
pub fn with_http_client<F, R>(f: F) -> Result<R, String>
where
    F: FnOnce(&ApiClient) -> R,
{
    match GLOBAL_CLIENT.lock() {
        Ok(client) => Ok(f(&client)),
        Err(e) => Err(format!("Failed to get client: {}", e)),
    }
}
