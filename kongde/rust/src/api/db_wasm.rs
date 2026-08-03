// WASM KV — 用浏览器 localStorage 持久化（web 部署下服务器配置/token 不丢失）
// 与 native 的 db.rs 格式一致：int/double 均存文本

#[cfg(target_arch = "wasm32")]
fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

#[flutter_rust_bridge::frb]
#[derive(Debug, Clone)]
pub struct LocalSong {
    pub path: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: String,
    pub cover_path: String,
    pub album_id: String,
    pub primary_color: i64,
    pub secondary_color: i64,
}

#[allow(unused_variables)]
pub fn init_db(db_path: String) -> Result<bool, String> {
    Ok(true)
}

pub fn kv_get(key: String) -> Result<Option<String>, String> {
    Ok(storage().and_then(|s| s.get_item(&key).ok().flatten()))
}
pub fn kv_set(key: String, value: String) -> Result<(), String> {
    if let Some(s) = storage() {
        let _ = s.set_item(&key, &value);
    }
    Ok(())
}
pub fn kv_delete(key: String) -> Result<bool, String> {
    match storage() {
        Some(s) => {
            let _ = s.remove_item(&key);
            Ok(true)
        }
        None => Ok(false),
    }
}
pub fn kv_keys() -> Result<Vec<String>, String> {
    Ok(match storage() {
        Some(s) => (0..s.length().unwrap_or(0))
            .filter_map(|i| s.key(i).ok().flatten())
            .collect(),
        None => vec![],
    })
}
pub fn kv_get_int(key: String) -> Result<Option<i64>, String> {
    match kv_get(key)? {
        Some(s) => s
            .parse::<i64>()
            .map(Some)
            .map_err(|e| format!("kv 值无法解析为整数: {}", e)),
        None => Ok(None),
    }
}
pub fn kv_set_int(key: String, value: i64) -> Result<(), String> {
    kv_set(key, value.to_string())
}
pub fn kv_get_double(key: String) -> Result<Option<f64>, String> {
    match kv_get(key)? {
        Some(s) => s
            .parse::<f64>()
            .map(Some)
            .map_err(|e| format!("kv 值无法解析为浮点数: {}", e)),
        None => Ok(None),
    }
}
pub fn kv_set_double(key: String, value: f64) -> Result<(), String> {
    kv_set(key, value.to_string())
}
pub fn kv_json_get(key: String) -> Result<Option<String>, String> {
    kv_get(key)
}
pub fn kv_json_set(key: String, json: String) -> Result<(), String> {
    kv_set(key, json)
}
pub fn kv_clear() -> Result<(), String> {
    if let Some(s) = storage() {
        let keys: Vec<String> = kv_keys().unwrap_or_default();
        for k in keys {
            let _ = s.remove_item(&k);
        }
    }
    Ok(())
}

pub fn get_local_songs() -> Result<Vec<LocalSong>, String> {
    Ok(vec![])
}
pub fn import_local_songs(
    paths: Vec<String>,
    covers_dir: String,
) -> Result<Vec<LocalSong>, String> {
    Ok(vec![])
}
pub fn clear_local_songs() -> Result<(), String> {
    Ok(())
}
