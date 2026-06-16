use super::client::ApiClient;

/// 根据汉字获取拼音
pub async fn get_pinyin_by_ori(
    client: &ApiClient,
    ori: &str,
) -> Result<Vec<String>, reqwest::Error> {
    let path = format!("/api/pinyin/get-by-ori?ori={}", ori);
    let response = client.get(&path).await?;
    response.json().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_pinyin_by_ori() {
        let client = ApiClient::default();

        match get_pinyin_by_ori(&client, "你好").await {
            Ok(pinyin_list) => {
                println!("根据汉字获取拼音成功！响应: {:?}", pinyin_list);
            }
            Err(error) => {
                println!("根据汉字获取拼音失败: {}", error);
            }
        }
    }
}
