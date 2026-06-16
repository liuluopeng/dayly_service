use flutter_rust_bridge::frb;

use crate::api::{logger_bridge::log_to_dart, wifi_api::init::get_client_clone};

pub use common::api::{
    base::{ApiError, ApiResponse},
    client::ApiClient,
    user::{LoginResponse, UpdateUserSettings, UserSettings, get_user_settings, update_user_settings, user_login},
};

#[frb(mirror(LoginResponse))]
pub struct _LoginResponse {
    pub token: String,
    pub token_type: String,
}

#[frb(mirror(UserSettings))]
pub struct _UserSettings {
    pub language: String,
    pub flutter_theme: String,
}

#[frb(mirror(UpdateUserSettings))]
pub struct _UpdateUserSettings {
    pub language: Option<String>,
    pub flutter_theme: Option<String>,
}

pub async fn user_login_for_dart(
    username: &str,
    password: &str,
) -> Result<LoginResponse, ApiError> {
    let mut client = get_client_clone().map_err(|e| ApiError::Internal(e.to_string()))?;

    match user_login(&mut client, username, password).await {
        Ok(res) => {
            if let Some(login_response) = res.data {
                // 登录成功后，将token保存到全局客户端中
                if let Err(e) = crate::api::wifi_api::init::set_client_token(&login_response.token)
                {
                    eprintln!("Failed to set client token: {}", e);
                }
                log_to_dart(format!("登录成功: {}", username));

                Ok(login_response)
            } else {
                Err(ApiError::Internal("No data found in response".to_string()))
            }
        }
        Err(err) => Err(err),
    }
}

pub async fn get_user_settings_for_dart() -> Result<UserSettings, ApiError> {
    let client = get_client_clone().map_err(|e| ApiError::Internal(e.to_string()))?;

    match get_user_settings(&client).await {
        Ok(res) => {
            if let Some(settings) = res.data {
                Ok(settings)
            } else {
                Err(ApiError::Internal("No data found in response".to_string()))
            }
        }
        Err(err) => Err(err),
    }
}

pub async fn update_user_settings_for_dart(
    language: Option<String>,
    flutter_theme: Option<String>,
) -> Result<UserSettings, ApiError> {
    let client = get_client_clone().map_err(|e| ApiError::Internal(e.to_string()))?;

    let settings = UpdateUserSettings {
        language,
        flutter_theme,
    };

    match update_user_settings(&client, &settings).await {
        Ok(res) => {
            if let Some(settings) = res.data {
                Ok(settings)
            } else {
                Err(ApiError::Internal("No data found in response".to_string()))
            }
        }
        Err(err) => Err(err),
    }
}
