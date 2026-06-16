use common::front_can_do::password;

/// 生成随机密码
#[flutter_rust_bridge::frb(sync)]
pub fn generate_password(length: usize) -> String {
    password::generate_password(length)
}

/// 生成强密码
#[flutter_rust_bridge::frb(sync)]
pub fn generate_strong_password(length: usize) -> String {
    password::generate_strong_password(length)
}
