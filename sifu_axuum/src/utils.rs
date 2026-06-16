use common::api::base::ApiError;

use crate::middleware::{AuthError, get_uid_from_token};
use axum::http::HeaderMap;

/// 从请求头中提取 UID
pub fn extract_uid_from_header(headers: &HeaderMap, jwt_secret: &str) -> Result<String, ApiError> {
    let auth_header = headers
        .get(axum::http::header::AUTHORIZATION)
        .ok_or(ApiError::unauthorized(ApiError::INVALID_TOKEN))?
        .to_str()
        .map_err(|_| ApiError::unauthorized(ApiError::INVALID_TOKEN))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(ApiError::unauthorized(ApiError::INVALID_TOKEN))?;

    let uid = get_uid_from_token(token, jwt_secret)
        .map_err(|_| ApiError::unauthorized(ApiError::INVALID_TOKEN))?;

    Ok(uid)
}

impl From<AuthError> for ApiError {
    fn from(_err: AuthError) -> Self {
        ApiError::unauthorized(ApiError::INVALID_TOKEN)
    }
}
