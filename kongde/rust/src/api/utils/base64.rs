use common::front_can_do::base64::{base64_decode, base64_encode};

/// 将字符串编码为 base64 格式
#[flutter_rust_bridge::frb(sync)]
pub fn base64_encode_wasm(input: &str) -> String {
    base64_encode(input)
}

/// 将 base64 编码的字符串解码为原始字符串
/// 如果解码失败，返回 None
#[flutter_rust_bridge::frb(sync)]
pub fn base64_decode_wasm(input: &str) -> Option<String> {
    base64_decode(input)
}
