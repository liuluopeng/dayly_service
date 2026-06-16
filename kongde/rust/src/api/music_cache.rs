use crate::api::logger_bridge::log_to_dart;
use common::read_metadata::{AudioMetadata, read_metadata};
use std::path::Path;
use std::time::SystemTime;

fn base64_encode(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data)
}

fn base64_decode(s: &str) -> Option<Vec<u8>> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.decode(s).ok()
}

/// 带缓存的元数据读取，使用共享的 settings.db pool
pub fn read_metadata_cached(file_path: String) -> Result<AudioMetadata, String> {
    let path = Path::new(&file_path);
    let modified = path
        .metadata()
        .map(|m| m.modified().unwrap_or(SystemTime::UNIX_EPOCH))
        .map(|t| t.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_secs())
        .unwrap_or(0) as i64;

    let rt = crate::api::runtime::shared_rt();
    rt.block_on(async {
        let p = crate::api::db::pool().map_err(|e| e)?;

        // 查缓存
        let cached: Option<(Option<String>, Option<String>, Option<String>, Option<i64>, Option<String>)> =
            sqlx::query_as("SELECT title, artist, album, duration_ms, picture_base64 FROM music_cache WHERE file_path = ? AND modified_secs = ?")
                .bind(&file_path).bind(modified)
                .fetch_optional(p).await.map_err(|e| format!("缓存查询失败: {}", e))?;

        if let Some((title, artist, album, duration_ms, picture_base64)) = cached {
            let picture = picture_base64.and_then(|b64| base64_decode(&b64));
            return Ok(AudioMetadata { title, artist, album, duration_ms: duration_ms.map(|d| d as u32), picture });
        }

        // 读文件
        let meta = read_metadata(&file_path)?;

        // 存缓存
        let picture_b64 = meta.picture.as_ref().and_then(|p| Some(base64_encode(p)));
        let _ = sqlx::query("INSERT OR REPLACE INTO music_cache (file_path, modified_secs, title, artist, album, duration_ms, picture_base64) VALUES (?, ?, ?, ?, ?, ?, ?)")
            .bind(&file_path).bind(modified)
            .bind(&meta.title).bind(&meta.artist).bind(&meta.album)
            .bind(meta.duration_ms.map(|d| d as i64))
            .bind(&picture_b64)
            .execute(p).await;

        Ok(meta)
    })
}
