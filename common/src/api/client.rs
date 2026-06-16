#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;

use reqwest::Client;

use super::base::{ApiError, ApiResponse, ErrorBody};

/// 统一的 API 客户端
/// 支持 native 和 wasm 两种平台
pub struct ApiClient {
    pub client: Client,
    pub base_url: String,
    pub token: Option<String>,
}

impl ApiClient {
    /// 创建新的 API 客户端
    pub fn new(base_url: &str) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        #[cfg(target_arch = "wasm32")]
        let client = Client::builder()
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: base_url.to_string(),
            token: None,
        }
    }

    /// 创建默认的 API 客户端（使用默认基础 URL）
    pub fn default() -> Self {
        Self::new("http://localhost:23001")
    }

    /// 获取基础 URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// 设置基础 URL
    pub fn set_base_url(&mut self, base_url: &str) {
        self.base_url = base_url.to_string();
    }

    /// 设置端口
    pub fn set_port(&mut self, port: u16) {
        // 找到协议部分结束的位置 (http:// 或 https://)
        let protocol_end = if self.base_url.starts_with("https://") {
            8
        } else if self.base_url.starts_with("http://") {
            7
        } else {
            0
        };

        // 找到路径开始的位置
        let path_start = self.base_url[protocol_end..]
            .find('/')
            .map(|pos| protocol_end + pos)
            .unwrap_or(self.base_url.len());

        // 找到端口分隔符 ':'，确保它在协议部分之后
        if let Some(port_start) = self.base_url[protocol_end..path_start].rfind(':') {
            let actual_port_start = protocol_end + port_start;

            // 替换端口
            let new_base = format!(
                "{}:{}{}",
                &self.base_url[..actual_port_start],
                port,
                &self.base_url[path_start..]
            );
            self.base_url = new_base;
        } else {
            // 没有端口，添加端口
            let new_base = format!(
                "{}:{}{}",
                &self.base_url[..path_start],
                port,
                &self.base_url[path_start..]
            );
            self.base_url = new_base;
        }


        
    }

    /// 获取 token
    pub fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    /// 设置 token
    pub fn set_token(&mut self, token: &str) {
        self.token = Some(token.to_string());
    }

    /// 清除 token
    pub fn clear_token(&mut self) {
        self.token = None;
    }

    pub fn tester_client() -> Self {
        let mut client = Self::default();
        client.set_token("eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1dWlkIjoiYTg2MTFlNDMtMzNlNC00MThmLWFhZTMtNGQ3NWY1OTQxZTk4IiwiZXhwIjoyMDAwMDAwMDAwfQ.iHHkGDpGsN_XpehSTGzmXCJMPnW2PvVd2UzihrAfly4");
        client
    }
}

impl ApiClient {
    /// Native 平台的 GET 请求
    pub async fn get(&self, path: &str) -> Result<reqwest::Response, reqwest::Error> {
        let url = format!("{}{}", self.base_url, path);
        #[cfg(not(target_arch = "wasm32"))]
        let mut request = self.client.get(&url).timeout(Duration::from_secs(10));

        #[cfg(target_arch = "wasm32")]
        let mut request = self.client.get(&url);

        // 如果有 token，添加到请求头
        if let Some(token) = self.token.as_ref() {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        request.send().await
    }

    /// Native 平台的 POST 请求
    pub async fn post(
        &self,
        path: &str,
        body: impl serde::Serialize,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let url = format!("{}{}", self.base_url, path);
        #[cfg(not(target_arch = "wasm32"))]
        let mut request = self
            .client
            .post(&url)
            .json(&body)
            .timeout(Duration::from_secs(10));

        #[cfg(target_arch = "wasm32")]
        let mut request = self.client.post(&url).json(&body);

        // 如果有 token，添加到请求头
        if let Some(token) = self.token.as_ref() {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        request.send().await
    }

    /// Native 平台的 PUT 请求
    pub async fn put(
        &self,
        path: &str,
        body: impl serde::Serialize,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let url = format!("{}{}", self.base_url, path);
        #[cfg(not(target_arch = "wasm32"))]
        let mut request = self
            .client
            .put(&url)
            .json(&body)
            .timeout(Duration::from_secs(10));

        #[cfg(target_arch = "wasm32")]
        let mut request = self.client.put(&url).json(&body);

        // 如果有 token，添加到请求头
        if let Some(token) = self.token.as_ref() {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        request.send().await
    }

    /// 统一的 JSON 响应处理：成功时解析为 ApiResponse<T>，失败时解析 ErrorBody 为 ApiError
    pub async fn json_response<T: serde::de::DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<ApiResponse<T>, ApiError> {
        let status = response.status();
        if status.is_success() {
            response
                .json::<ApiResponse<T>>()
                .await
                .map_err(|e| ApiError::Internal(format!("响应解析失败: {}", e)))
        } else {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "无法读取响应体".to_string());
            // 尝试解析为 ErrorBody
            if let Ok(error_body) = serde_json::from_str::<ErrorBody>(&body) {
                Err(error_body.to_api_error())
            } else {
                Err(ApiError::Internal(format!("HTTP {}: {}", status, body)))
            }
        }
    }

    /// Native 平台的 DELETE 请求
    pub async fn delete(&self, path: &str) -> Result<reqwest::Response, reqwest::Error> {
        let url = format!("{}{}", self.base_url, path);
        #[cfg(not(target_arch = "wasm32"))]
        let mut request = self.client.delete(&url).timeout(Duration::from_secs(10));

        #[cfg(target_arch = "wasm32")]
        let mut request = self.client.delete(&url);

        // 如果有 token，添加到请求头
        if let Some(token) = self.token.as_ref() {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        request.send().await
    }
}
