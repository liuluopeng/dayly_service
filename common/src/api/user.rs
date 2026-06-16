use super::base::{ApiError, ApiResponse, ApiResult};
use super::client::ApiClient;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserSettings {
    pub language: String,
    pub flutter_theme: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserSettings {
    pub language: Option<String>,
    pub flutter_theme: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub token_type: String,
}

impl fmt::Display for LoginResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const RESET: &str = "\x1b[0m";
        const GREEN: &str = "\x1b[32m";
        const CYAN: &str = "\x1b[36m";

        write!(f, "{}{}:{}{}\n", GREEN, "Token", RESET, self.token)?;
        write!(f, "{}{}:{}{}", CYAN, "Token Type", RESET, self.token_type)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

impl fmt::Display for LoginRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const RESET: &str = "\x1b[0m";
        const GREEN: &str = "\x1b[32m";
        const CYAN: &str = "\x1b[36m";

        write!(f, "{}{}:{}{}\n", GREEN, "Username", RESET, self.username)?;
        write!(f, "{}{}:{}{}", CYAN, "Password", RESET, "******")
    }
}

/// 用户登录
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
pub async fn get_user_settings(
    client: &ApiClient,
) -> ApiResult<ApiResponse<UserSettings>> {
    let response = client
        .get("/api/user/settings")
        .await
        .map_err(|e| ApiError::Internal(format!("{}", e)))?;
    client.json_response::<UserSettings>(response).await
}

/// 更新用户设置
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
    let client = ApiClient::tester_client();
    let username = "admin";
    let password = "qazwsx";

    let result = user_login(&client, username, password).await.unwrap();

    println!("{}", result);
}
