use super::client::ApiClient;

/// 向指定地址发送 GET 请求
///
/// # 参数
/// - `client`: API 客户端实例
///
/// # 返回值
/// - `Result<String, reqwest::Error>`: 请求成功返回响应体，失败返回错误
pub async fn send_hello_request(client: &ApiClient) -> Result<String, reqwest::Error> {
    let response = client.get("/hello").await?;
    response.text().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hello_endpoint() {
        let client = ApiClient::default();

        match send_hello_request(&client).await {
            Ok(response) => {
                println!("请求成功！响应: {}", response);
            }
            Err(error) => {
                println!("请求失败: {}", error);
            }
        }
    }
}

