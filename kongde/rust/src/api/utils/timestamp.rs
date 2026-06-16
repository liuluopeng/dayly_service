use common::front_can_do::timestamp;

/// 将时间戳转换为本地时间字符串
pub fn timestamp_to_local(timestamp: i64) -> String {
    timestamp::timestamp_to_local(timestamp)
}

/// 将时间戳转换为UTC时间字符串
pub fn timestamp_to_utc(timestamp: i64) -> String {
    timestamp::timestamp_to_utc(timestamp)
}

/// 将本地时间字符串转换为时间戳
pub fn local_to_timestamp(time_str: &str) -> Option<i64> {
    timestamp::local_to_timestamp(time_str)
}

/// 获取当前时间戳
pub fn get_current_timestamp() -> i64 {
    timestamp::get_current_timestamp()
}

/// 获取当前本地时间字符串
pub fn get_current_local_time() -> String {
    timestamp::get_current_local_time()
}

/// 获取当前UTC时间字符串
pub fn get_current_utc_time() -> String {
    timestamp::get_current_utc_time()
}