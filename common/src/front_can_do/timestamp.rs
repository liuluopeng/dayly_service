use chrono::{DateTime, Local, Utc};

/// 将时间戳转换为本地时间字符串
/// # Arguments
/// * `timestamp` - 时间戳（秒）
/// # Returns
/// * 本地时间字符串，格式为 "YYYY-MM-DD HH:MM:SS"
pub fn timestamp_to_local(timestamp: i64) -> String {
    let datetime: DateTime<Utc> =
        DateTime::from_timestamp(timestamp, 0).unwrap_or_else(|| Utc::now());
    let local_datetime: DateTime<Local> = datetime.into();
    local_datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// 将时间戳转换为 UTC 时间字符串
/// # Arguments
/// * `timestamp` - 时间戳（秒）
/// # Returns
/// * UTC 时间字符串，格式为 "YYYY-MM-DD HH:MM:SS UTC"
pub fn timestamp_to_utc(timestamp: i64) -> String {
    let datetime: DateTime<Utc> =
        DateTime::from_timestamp(timestamp, 0).unwrap_or_else(|| Utc::now());
    datetime.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

/// 将本地时间字符串转换为时间戳
/// # Arguments
/// * `time_str` - 本地时间字符串，格式为 "YYYY-MM-DD HH:MM:SS"
/// # Returns
/// * 时间戳（秒），如果解析失败则返回 None
pub fn local_to_timestamp(time_str: &str) -> Option<i64> {
    use chrono::NaiveDateTime;
    let naive_datetime = NaiveDateTime::parse_from_str(time_str, "%Y-%m-%d %H:%M:%S").ok()?;
    let datetime = naive_datetime.and_local_timezone(Local).earliest()?;
    Some(datetime.timestamp())
}

/// 获取当前时间戳（秒）
/// # Returns
/// * 当前时间戳（秒）
pub fn get_current_timestamp() -> i64 {
    Utc::now().timestamp()
}

/// 获取当前本地时间字符串
/// # Returns
/// * 当前本地时间字符串，格式为 "YYYY-MM-DD HH:MM:SS"
pub fn get_current_local_time() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

/// 获取当前 UTC 时间字符串
/// # Returns
/// * 当前 UTC 时间字符串，格式为 "YYYY-MM-DD HH:MM:SS UTC"
pub fn get_current_utc_time() -> String {
    Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string()
}
