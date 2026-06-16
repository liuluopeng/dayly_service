use common::front_can_do::uuid;

/// 生成UUID v4
pub fn generate_uuid_v4() -> String {
    uuid::generate_uuid_v4()
}

/// 生成UUID v5
pub fn generate_uuid_v5(namespace: &str, name: &str) -> String {
    uuid::generate_uuid_v5(namespace, name)
}

/// 生成UUID v6
pub fn generate_uuid_v6() -> String {
    uuid::generate_uuid_v6()
}

/// 生成UUID v7
pub fn generate_uuid_v7() -> String {
    uuid::generate_uuid_v7()
}

/// 验证UUID格式
pub fn validate_uuid(uuid_str: &str) -> bool {
    uuid::validate_uuid(uuid_str)
}