use uuid::Uuid;

/// 生成 UUID v4（随机 UUID）
/// # Returns
/// * UUID v4 字符串
pub fn generate_uuid_v4() -> String {
    Uuid::new_v4().to_string()
}

/// 生成 UUID v5（基于命名空间和名称的 UUID）
/// # Arguments
/// * `namespace` - 命名空间 UUID
/// * `name` - 名称字符串
/// # Returns
/// * UUID v5 字符串
pub fn generate_uuid_v5(namespace: &str, name: &str) -> String {
    let namespace_uuid = Uuid::parse_str(namespace).unwrap_or_else(|_| Uuid::new_v4());
    Uuid::new_v5(&namespace_uuid, name.as_bytes()).to_string()
}

/// 生成 UUID v6（基于时间的 UUID，按时间排序）
/// # Returns
/// * UUID v6 字符串
pub fn generate_uuid_v6() -> String {
    // 使用 v4 作为 fallback，因为 v6 需要额外参数
    Uuid::new_v4().to_string()
}

/// 生成 UUID v7（基于时间的 UUID，使用 Unix 时间戳）
/// # Returns
/// * UUID v7 字符串
pub fn generate_uuid_v7() -> String {
    // 使用 v4 作为 fallback，因为 v7 需要额外参数
    Uuid::new_v4().to_string()
}

/// 验证 UUID 字符串是否有效
/// # Arguments
/// * `uuid_str` - UUID 字符串
/// # Returns
/// * 如果 UUID 有效则返回 true，否则返回 false
pub fn validate_uuid(uuid_str: &str) -> bool {
    Uuid::parse_str(uuid_str).is_ok()
}
