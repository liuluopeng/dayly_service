// 字词学习——数据在服务端 sqlite，通过 API 查询（与 Vue/wasm-demo 同源）

use crate::api::wifi_api::init::get_client_clone;

/// 生字表：按年级/学期（grade 1-6，term 1-2）
pub async fn zici_new_chars(grade: u32, term: u32) -> Result<Vec<String>, String> {
    let client = get_client_clone()?;
    let resp = common::api::zici::zici_chars(&client, grade, term)
        .await
        .map_err(|e| format!("{}", e))?;
    Ok(resp.data.unwrap_or_default())
}

/// 生词表：搜索（返回最多 500 个）
pub async fn zici_new_words(query: String) -> Result<Vec<String>, String> {
    let client = get_client_clone()?;
    let resp = common::api::zici::zici_words(&client, &query, 1, 500)
        .await
        .map_err(|e| format!("{}", e))?;
    Ok(resp
        .data
        .and_then(|v| v["data"].as_array().cloned())
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect())
}

/// 词频搜索（含释义）
#[flutter_rust_bridge::frb]
pub struct WordFrequencyEntry {
    pub word: String,
    pub pinyin: String,
    pub frequency: u32,
    pub explanation: String,
}

pub async fn zici_word_frequency_search(
    query: String,
    limit: u32,
) -> Result<Vec<WordFrequencyEntry>, String> {
    let client = get_client_clone()?;
    let resp = common::api::zici::zici_word_frequency(&client, &query, limit)
        .await
        .map_err(|e| format!("{}", e))?;
    Ok(resp
        .data
        .unwrap_or_default()
        .into_iter()
        .map(|v| WordFrequencyEntry {
            word: v["word"].as_str().unwrap_or("").to_string(),
            pinyin: v["pinyin"].as_str().unwrap_or("").to_string(),
            frequency: v["frequency"].as_u64().unwrap_or(0) as u32,
            explanation: v["explanation"].as_str().unwrap_or("").to_string(),
        })
        .collect())
}
