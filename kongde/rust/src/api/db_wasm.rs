// WASM stub — SQLite 不可用，全部返回默认值

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
pub fn init_db(db_path: String) -> Result<bool, String> { Ok(true) }

pub fn kv_get(key: String) -> Result<Option<String>, String> { Ok(None) }
pub fn kv_set(key: String, value: String) -> Result<(), String> { Ok(()) }
pub fn kv_delete(key: String) -> Result<bool, String> { Ok(false) }
pub fn kv_keys() -> Result<Vec<String>, String> { Ok(vec![]) }
pub fn kv_get_int(key: String) -> Result<Option<i64>, String> { Ok(None) }
pub fn kv_set_int(key: String, value: i64) -> Result<(), String> { Ok(()) }
pub fn kv_get_double(key: String) -> Result<Option<f64>, String> { Ok(None) }
pub fn kv_set_double(key: String, value: f64) -> Result<(), String> { Ok(()) }
pub fn kv_json_get(key: String) -> Result<Option<String>, String> { Ok(None) }
pub fn kv_json_set(key: String, json: String) -> Result<(), String> { Ok(()) }
pub fn kv_clear() -> Result<(), String> { Ok(()) }

pub fn get_local_songs() -> Result<Vec<LocalSong>, String> { Ok(vec![]) }
pub fn import_local_songs(paths: Vec<String>, covers_dir: String) -> Result<Vec<LocalSong>, String> { Ok(vec![]) }
pub fn clear_local_songs() -> Result<(), String> { Ok(()) }
