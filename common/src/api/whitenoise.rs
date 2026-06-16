use super::client::ApiClient;

/// 白噪音音频文件结构体
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AudioFile {
    pub name: String,
    pub url: String,
}

/// 获取白噪音音频文件列表
pub async fn get_whitenoise_list(client: &ApiClient) -> Result<Vec<AudioFile>, reqwest::Error> {
    let response = client.get("/whitenoise/list").await?;
    response.json().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_whitenoise_list() {
        let client = ApiClient::default();

        match get_whitenoise_list(&client).await {
            Ok(audio_files) => {
                println!("获取白噪音音频文件列表成功！响应: {:?}", audio_files);
            }
            Err(error) => {
                println!("获取白噪音音频文件列表失败: {}", error);
            }
        }
    }
}

