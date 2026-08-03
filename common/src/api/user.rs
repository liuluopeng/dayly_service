use super::base::{ApiError, ApiResponse, ApiResult};
use super::client::ApiClient;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize, Clone)]
/// 用户设置
#[allow(missing_docs)]
pub struct UserSettings {
    pub language: String,
    pub flutter_theme: String,
}

#[derive(Debug, Serialize, Deserialize)]
/// 更新用户设置的请求体
#[allow(missing_docs)]
pub struct UpdateUserSettings {
    pub language: Option<String>,
    pub flutter_theme: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
/// 登录响应（token）
#[allow(missing_docs)]
pub struct LoginResponse {
    pub token: String,
    pub token_type: String,
}

impl fmt::Display for LoginResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const RESET: &str = "\x1b[0m";
        const GREEN: &str = "\x1b[32m";
        const CYAN: &str = "\x1b[36m";

        writeln!(f, "{}Token:{}{}", GREEN, RESET, self.token)?;
        write!(f, "{}Token Type:{}{}", CYAN, RESET, self.token_type)
    }
}

#[derive(Debug, Deserialize, Serialize)]
/// 登录请求体
#[allow(missing_docs)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

impl fmt::Display for LoginRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const RESET: &str = "\x1b[0m";
        const GREEN: &str = "\x1b[32m";
        const CYAN: &str = "\x1b[36m";

        writeln!(f, "{}Username:{}{}", GREEN, RESET, self.username)?;
        write!(f, "{}Password:{}******", CYAN, RESET)
    }
}

/// 用户登录
/// 用户登录，返回 JWT token
pub async fn user_login(
    client: &ApiClient,
    username: &str,
    password: &str,
) -> ApiResult<ApiResponse<LoginResponse>> {
    let request = serde_json::json!({
        "username": username,
        "password": password,
    });
    let response = client
        .post("/api/user/login", &request)
        .await
        .map_err(|e| ApiError::Internal(format!("{}", e)))?;
    client.json_response::<LoginResponse>(response).await
}

/// 获取用户设置
/// 获取当前用户设置
pub async fn get_user_settings(client: &ApiClient) -> ApiResult<ApiResponse<UserSettings>> {
    let response = client
        .get("/api/user/settings")
        .await
        .map_err(|e| ApiError::Internal(format!("{}", e)))?;
    client.json_response::<UserSettings>(response).await
}

/// 更新用户设置
/// 更新当前用户设置
pub async fn update_user_settings(
    client: &ApiClient,
    settings: &UpdateUserSettings,
) -> ApiResult<ApiResponse<UserSettings>> {
    let response = client
        .put("/api/user/settings", settings)
        .await
        .map_err(|e| ApiError::Internal(format!("{}", e)))?;
    client.json_response::<UserSettings>(response).await
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn test_user_login() {
    let client = ApiClient::test_client();
    let username = "admin";
    let password = "qazwsx";

    let result = user_login(&client, username, password).await.unwrap();

    println!("{}", result);
}
