use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Debug;

use thiserror::Error;

// 仅在非 WebAssembly 环境中导入 axum
#[cfg(not(target_arch = "wasm32"))]
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

// 在 WebAssembly 环境中提供空实现
#[cfg(target_arch = "wasm32")]
pub trait IntoResponse {
    fn into_response(self) -> ();
}

#[cfg(target_arch = "wasm32")]
impl<T> IntoResponse for T {
    fn into_response(self) -> () {
        ()
    }
}

#[cfg(target_arch = "wasm32")]
pub type Response = ();

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ApiResponse<T> {
    pub msg: String,
    pub data: Option<T>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: i64,
}

impl<T> PaginatedResponse<T>
where
    T: Debug,
{
    /// 打印带有颜色的分页响应
    pub fn print_colored(&self) {
        // ANSI 颜色代码
        const RESET: &str = "\x1b[0m";
        const CYAN: &str = "\x1b[36m";
        const GREEN: &str = "\x1b[32m";
        const YELLOW: &str = "\x1b[33m";
        const MAGENTA: &str = "\x1b[35m";
        const BLUE: &str = "\x1b[34m";

        println!("{}{}:{}{}", CYAN, "总条数", RESET, self.total);
        println!("{}{}:{}{}", GREEN, "当前页码", RESET, self.page);
        println!("{}{}:{}{}", YELLOW, "每页大小", RESET, self.page_size);
        println!("{}{}:{}{}", MAGENTA, "总页数", RESET, self.total_pages);
        println!("{}{}:{}{:?}", BLUE, "数据", RESET, self.data);
    }
}
impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            msg: "ok".into(),
            data: Some(data),
        }
    }

    pub fn fail(msg: impl Into<String>) -> Self {
        Self {
            msg: msg.into(),
            data: None,
        }
    }
}

impl<T> fmt::Display for ApiResponse<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // ANSI 颜色代码
        const RESET: &str = "\x1b[0m";
        const GREEN: &str = "\x1b[32m";
        const BLUE: &str = "\x1b[34m";

        write!(f, "{}{}:{}{}\n", GREEN, "消息", RESET, self.msg)?;
        if let Some(ref data) = self.data {
            write!(f, "{}{}:{}\n{}", BLUE, "数据", RESET, data)?;
        } else {
            write!(f, "{}{}:{}{}", BLUE, "数据", RESET, "无")?;
        }
        Ok(())
    }
}

impl<T> fmt::Display for PaginatedResponse<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // ANSI 颜色代码
        const RESET: &str = "\x1b[0m";
        const CYAN: &str = "\x1b[36m";
        const GREEN: &str = "\x1b[32m";
        const YELLOW: &str = "\x1b[33m";
        const MAGENTA: &str = "\x1b[35m";
        const BLUE: &str = "\x1b[34m";

        write!(f, "{}{}:{}{}\n", CYAN, "总条数", RESET, self.total)?;
        write!(f, "{}{}:{}{}\n", GREEN, "当前页码", RESET, self.page)?;
        write!(f, "{}{}:{}{}\n", YELLOW, "每页大小", RESET, self.page_size)?;
        write!(f, "{}{}:{}{}\n", MAGENTA, "总页数", RESET, self.total_pages)?;
        write!(f, "{}{}:{}\n", BLUE, "数据", RESET)?;
        for (i, item) in self.data.iter().enumerate() {
            write!(f, "{}{}{}:\n{}\n", "  ", i + 1, ".", item)?;
        }
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<T> IntoResponse for ApiResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

#[derive(Debug, Error, Serialize, Deserialize, Clone)]
pub enum ApiError {
    #[error("Bad request: {message}")]
    BadRequest { code: String, message: String },

    #[error("Not found: {message}")]
    NotFound { code: String, message: String },

    #[error("Unauthorized")]
    Unauthorized { code: String },

    #[error("Forbidden: {message}")]
    Forbidden { code: String, message: String },

    #[error("Internal error: {0}")]
    Internal(String),
}

// 业务错误码常量
impl ApiError {
    // ---- BadRequest ----
    pub const EMPTY_CREDENTIALS: &'static str = "EMPTY_CREDENTIALS";
    pub const EMPTY_PATH: &'static str = "EMPTY_PATH";
    pub const INVALID_MEDIA_TYPE: &'static str = "INVALID_MEDIA_TYPE";
    pub const NO_AUTHORIZED_DIR: &'static str = "NO_AUTHORIZED_DIR";
    pub const PATH_NOT_IN_DIR: &'static str = "PATH_NOT_IN_DIR";
    pub const NOT_A_DIRECTORY: &'static str = "NOT_A_DIRECTORY";
    pub const OPENAI_KEY_MISSING: &'static str = "OPENAI_KEY_MISSING";

    // ---- NotFound ----
    pub const USER_NOT_FOUND: &'static str = "USER_NOT_FOUND";
    pub const SONG_NOT_FOUND: &'static str = "SONG_NOT_FOUND";
    pub const SONG_COVER_NOT_FOUND: &'static str = "SONG_COVER_NOT_FOUND";
    pub const LYRICS_NOT_FOUND: &'static str = "LYRICS_NOT_FOUND";
    pub const NO_TTML_LYRICS: &'static str = "NO_TTML_LYRICS";
    pub const DIR_NOT_FOUND: &'static str = "DIR_NOT_FOUND";
    pub const CHARACTER_NOT_FOUND: &'static str = "CHARACTER_NOT_FOUND";
    pub const PATH_NOT_FOUND: &'static str = "PATH_NOT_FOUND";
    pub const SESSION_NOT_FOUND: &'static str = "SESSION_NOT_FOUND";
    pub const MEDIA_PATH_NOT_FOUND: &'static str = "MEDIA_PATH_NOT_FOUND";
    pub const SHORT_NOTE_NOT_FOUND: &'static str = "SHORT_NOTE_NOT_FOUND";
    pub const NOTE_NOT_FOUND: &'static str = "NOTE_NOT_FOUND";
    pub const NOTE_CONTENT_NOT_FOUND: &'static str = "NOTE_CONTENT_NOT_FOUND";
    pub const RESOURCE_NOT_FOUND: &'static str = "RESOURCE_NOT_FOUND";

    // ---- Unauthorized ----
    pub const WRONG_PASSWORD: &'static str = "WRONG_PASSWORD";
    pub const INVALID_TOKEN: &'static str = "INVALID_TOKEN";
    pub const TOKEN_EXPIRED: &'static str = "TOKEN_EXPIRED";

    // ---- Forbidden ----
    pub const ADMIN_REQUIRED: &'static str = "ADMIN_REQUIRED";
    pub const DIR_ACCESS_DENIED: &'static str = "DIR_ACCESS_DENIED";

    // ---- 便捷构造方法 ----
    pub fn bad_request(code: &str, message: impl Into<String>) -> Self {
        ApiError::BadRequest { code: code.to_string(), message: message.into() }
    }

    pub fn not_found(code: &str, message: impl Into<String>) -> Self {
        ApiError::NotFound { code: code.to_string(), message: message.into() }
    }

    pub fn unauthorized(code: &str) -> Self {
        ApiError::Unauthorized { code: code.to_string() }
    }

    pub fn forbidden(code: &str, message: impl Into<String>) -> Self {
        ApiError::Forbidden { code: code.to_string(), message: message.into() }
    }

    pub fn error_code(&self) -> &str {
        match self {
            ApiError::BadRequest { code, .. } => code,
            ApiError::NotFound { code, .. } => code,
            ApiError::Unauthorized { code } => code,
            ApiError::Forbidden { code, .. } => code,
            ApiError::Internal(_) => "INTERNAL_ERROR",
        }
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        ApiError::Internal(err.to_string())
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_type, code, message) = match &self {
            ApiError::BadRequest { code, message } => {
                eprintln!("Bad request: {:?}", message);
                (StatusCode::BAD_REQUEST, "bad_request", code.clone(), message.clone())
            }
            ApiError::NotFound { code, message } => {
                (StatusCode::NOT_FOUND, "not_found", code.clone(), message.clone())
            }
            ApiError::Unauthorized { code } => (
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                code.clone(),
                "unauthorized".into(),
            ),
            ApiError::Forbidden { code, message } => {
                (StatusCode::FORBIDDEN, "forbidden", code.clone(), message.clone())
            }
            ApiError::Internal(err) => {
                eprintln!("Error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "INTERNAL_ERROR".to_string(),
                    "internal server error".into(),
                )
            }
        };

        let body = ErrorBody {
            error: ErrorInfo {
                r#type: error_type.to_string(),
                code,
                message,
            },
        };

        (status, Json(body)).into_response()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorBody {
    pub error: ErrorInfo,
}

impl ErrorBody {
    pub fn to_api_error(&self) -> ApiError {
        let code = &self.error.code;
        match self.error.r#type.as_str() {
            "bad_request" => ApiError::BadRequest {
                code: code.clone(),
                message: self.error.message.clone(),
            },
            "not_found" => ApiError::NotFound {
                code: code.clone(),
                message: self.error.message.clone(),
            },
            "unauthorized" => ApiError::Unauthorized {
                code: code.clone(),
            },
            "forbidden" => ApiError::Forbidden {
                code: code.clone(),
                message: self.error.message.clone(),
            },
            "internal_error" => ApiError::Internal(self.error.message.clone()),
            _ => ApiError::Internal(self.error.message.clone()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorInfo {
    pub r#type: String,
    pub code: String,
    pub message: String,
}

impl fmt::Display for ErrorBody {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error)
    }
}

crate::impl_display!(self, ErrorInfo,
    "类型" => self.r#type,
    "代码" => self.code,
    "消息" => self.message
);

pub type ApiResult<T> = std::result::Result<T, ApiError>;
