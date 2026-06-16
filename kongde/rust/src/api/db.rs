use crate::api::logger_bridge::log_to_dart;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::path::Path;
use std::sync::OnceLock;

static POOL: OnceLock<SqlitePool> = OnceLock::new();

/// 初始化 / 打开 SQLite 数据库，返回是否为新创建
#[flutter_rust_bridge::frb]
pub fn init_db(db_path: String) -> Result<bool, String> {
    let path = Path::new(&db_path);
    let is_new = !path.exists();

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }

    let rt = crate::api::runtime::shared_rt();
    let pool = rt.block_on(async {
        let options = SqliteConnectOptions::new()
            .filename(&db_path)
            .create_if_missing(true);

        SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .map_err(|e| format!("连接 SQLite 失败: {}", e))
    })?;

    rt.block_on(async {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS kv_store (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
            )",
        )
        .execute(&pool)
        .await
        .map_err(|e| format!("创建表失败: {}", e))
    })?;

    let _ = POOL.set(pool);
    run_migrations()?;
    log_to_dart(format!("SQLite 已初始化: {} (新: {})", db_path, is_new));
    Ok(is_new)
}

pub(crate) fn pool() -> Result<&'static SqlitePool, String> {
    POOL.get().ok_or_else(|| "数据库未初始化".to_string())
}

#[flutter_rust_bridge::frb]
pub fn kv_set(key: String, value: String) -> Result<(), String> {
    let p = pool()?;
    crate::api::runtime::shared_rt().block_on(async {
        sqlx::query("INSERT OR REPLACE INTO kv_store (key, value, updated_at) VALUES (?, ?, strftime('%s','now'))")
            .bind(&key).bind(&value)
            .execute(p).await.map_err(|e| format!("写入失败: {}", e))?;
        Ok(())
    })
}

#[flutter_rust_bridge::frb]
pub fn kv_get(key: String) -> Result<Option<String>, String> {
    let p = pool()?;
    crate::api::runtime::shared_rt().block_on(async {
        sqlx::query_scalar::<_, String>("SELECT value FROM kv_store WHERE key = ?")
            .bind(&key)
            .fetch_optional(p).await.map_err(|e| format!("读取失败: {}", e))
    })
}

#[flutter_rust_bridge::frb]
pub fn kv_delete(key: String) -> Result<bool, String> {
    let p = pool()?;
    crate::api::runtime::shared_rt().block_on(async {
        let rows = sqlx::query("DELETE FROM kv_store WHERE key = ?")
            .bind(&key)
            .execute(p).await.map_err(|e| format!("删除失败: {}", e))?
            .rows_affected();
        Ok(rows > 0)
    })
}

#[flutter_rust_bridge::frb]
pub fn kv_keys() -> Result<Vec<String>, String> {
    let p = pool()?;
    crate::api::runtime::shared_rt().block_on(async {
        sqlx::query_scalar::<_, String>("SELECT key FROM kv_store ORDER BY updated_at DESC")
            .fetch_all(p).await.map_err(|e| format!("查询失败: {}", e))
    })
}

// 类型化的 getter/setter，消除 Dart 端字符串转换
#[flutter_rust_bridge::frb]
pub fn kv_get_int(key: String) -> Result<Option<i64>, String> {
    match kv_get(key)? {
        Some(s) => Ok(s.parse::<i64>().ok()),
        None => Ok(None),
    }
}

#[flutter_rust_bridge::frb]
pub fn kv_set_int(key: String, value: i64) -> Result<(), String> {
    kv_set(key, value.to_string())
}

#[flutter_rust_bridge::frb]
pub fn kv_get_double(key: String) -> Result<Option<f64>, String> {
    match kv_get(key)? {
        Some(s) => Ok(s.parse::<f64>().ok()),
        None => Ok(None),
    }
}

#[flutter_rust_bridge::frb]
pub fn kv_set_double(key: String, value: f64) -> Result<(), String> {
    kv_set(key, value.to_string())
}

#[flutter_rust_bridge::frb]
pub fn kv_json_set(key: String, json: String) -> Result<(), String> {
    kv_set(key, json)
}

#[flutter_rust_bridge::frb]
pub fn kv_json_get(key: String) -> Result<Option<String>, String> {
    kv_get(key)
}

#[flutter_rust_bridge::frb]
pub fn kv_clear() -> Result<(), String> {
    let p = pool()?;
    crate::api::runtime::shared_rt().block_on(async {
        sqlx::query("DELETE FROM kv_store").execute(p).await.map_err(|e| format!("清空失败: {}", e))?;
        Ok(())
    })
}

// 数据库 schema 版本迁移
const CURRENT_SCHEMA_VERSION: i64 = 3;

fn run_migrations() -> Result<(), String> {
    let p = pool()?;
    crate::api::runtime::shared_rt().block_on(async {
        // 确保 db_meta 表存在
        sqlx::query("CREATE TABLE IF NOT EXISTS db_meta (key TEXT PRIMARY KEY, value TEXT NOT NULL)")
            .execute(p).await.map_err(|e| format!("创建 db_meta 失败: {}", e))?;

        // 读取当前版本
        let version: i64 = sqlx::query_scalar(
            "SELECT CAST(value AS INTEGER) FROM db_meta WHERE key = 'schema_version'"
        ).fetch_optional(p).await.ok().flatten().unwrap_or(0);

        if version < CURRENT_SCHEMA_VERSION {
            log_to_dart(format!("迁移 schema: v{} -> v{}", version, CURRENT_SCHEMA_VERSION));

            if version < 1 {
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS local_songs (
                        path TEXT PRIMARY KEY, title TEXT NOT NULL, artist TEXT NOT NULL DEFAULT '',
                        album TEXT NOT NULL DEFAULT '', duration TEXT NOT NULL DEFAULT '',
                        cover_path TEXT NOT NULL DEFAULT '', album_id TEXT NOT NULL DEFAULT '',
                        primary_color INTEGER NOT NULL DEFAULT 0, secondary_color INTEGER NOT NULL DEFAULT 0
                    )"
                ).execute(p).await.map_err(|e| format!("创建 local_songs 失败: {}", e))?;
            }

            if version < 2 {
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS music_cache (
                        file_path TEXT PRIMARY KEY, modified_secs INTEGER NOT NULL,
                        title TEXT, artist TEXT, album TEXT, duration_ms INTEGER, picture_base64 TEXT
                    )"
                ).execute(p).await.map_err(|e| format!("创建 music_cache 失败: {}", e))?;
            }
            if version < 3 {
                sqlx::query("ALTER TABLE local_songs ADD COLUMN primary_color INTEGER NOT NULL DEFAULT 0")
                    .execute(p).await.ok();
                sqlx::query("ALTER TABLE local_songs ADD COLUMN secondary_color INTEGER NOT NULL DEFAULT 0")
                    .execute(p).await.ok();
            }

            sqlx::query("INSERT OR REPLACE INTO db_meta (key, value) VALUES ('schema_version', ?)")
                .bind(CURRENT_SCHEMA_VERSION)
                .execute(p).await.map_err(|e| format!("更新 schema_version 失败: {}", e))?;
        }
        Ok(())
    })
}

// 本地歌曲表（SQLite 是唯一数据源）

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

#[flutter_rust_bridge::frb]
pub fn get_local_songs() -> Result<Vec<LocalSong>, String> {
    // table created by migration in init_db
    let p = pool()?;
    crate::api::runtime::shared_rt().block_on(async {
        sqlx::query_as::<_, (String, String, String, String, String, String, String, i64, i64)>(
            "SELECT path, title, artist, album, duration, cover_path, album_id, primary_color, secondary_color FROM local_songs ORDER BY title"
        ).fetch_all(p).await.map(|rows| {
            rows.into_iter().map(|(path, title, artist, album, duration, cover_path, album_id, primary_color, secondary_color)| {
                LocalSong { path, title, artist, album, duration, cover_path, album_id, primary_color, secondary_color }
            }).collect()
        }).map_err(|e| format!("查询失败: {}", e))
    })
}

fn simple_hash(s: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

/// 扫描并导入歌曲：Rust 读元数据 + 保存封面 + 写入 SQLite
#[flutter_rust_bridge::frb]
pub fn import_local_songs(paths: Vec<String>, covers_dir: String) -> Result<Vec<LocalSong>, String> {
    // table created by migration in init_db
    let p = pool()?;
    let total = paths.len();
    let mut imported = 0u32;
    let mut with_cover = 0u32;

    for path in paths {
        let meta = match common::read_metadata::read_metadata(&path) {
            Ok(m) => m,
            Err(_) => continue,
        };
        let dur_ms = meta.duration_ms.unwrap_or(0);
        let mins = dur_ms / 60000;
        let secs = (dur_ms % 60000) / 1000;
        let duration = format!("{:02}:{:02}", mins, secs);

        let cover_path = if let Some(ref pic) = meta.picture {
            if !pic.is_empty() {
                let dir = Path::new(&covers_dir);
                let _ = std::fs::create_dir_all(dir);
                let cp = dir.join(format!("{}.jpg", simple_hash(&path)));
                if std::fs::write(&cp, pic).is_ok() { with_cover += 1; cp.to_string_lossy().to_string() } else { String::new() }
            } else { String::new() }
        } else { String::new() };

        // 提取封面主色和次色
        let (primary_color, secondary_color) = if !cover_path.is_empty() {
            if let Ok(bytes) = std::fs::read(&cover_path) {
                common::color_extract::extract_colors(&bytes)
            } else { (None, None) }
        } else { (None, None) };

        let rt = crate::api::runtime::shared_rt();
        let _ = rt.block_on(async {
            sqlx::query("INSERT OR REPLACE INTO local_songs (path, title, artist, album, duration, cover_path, album_id, primary_color, secondary_color) VALUES (?, ?, ?, ?, ?, ?, '', ?, ?)")
                .bind(&path).bind(meta.title.unwrap_or_default()).bind(meta.artist.unwrap_or_default())
                .bind(meta.album.unwrap_or_default()).bind(&duration).bind(&cover_path)
                .bind(primary_color.unwrap_or(0)).bind(secondary_color.unwrap_or(0))
                .execute(p).await
        });
        imported += 1;
    }

    log_to_dart(format!("导入完成: {}/{} 首, {} 封面", imported, total, with_cover));
    get_local_songs()
}

#[flutter_rust_bridge::frb]
pub fn clear_local_songs() -> Result<(), String> {
    // table created by migration in init_db
    let p = pool()?;
    crate::api::runtime::shared_rt().block_on(async {
        sqlx::query("DELETE FROM local_songs").execute(p).await.map_err(|e| format!("清空失败: {}", e))?;
        Ok(())
    })
}

