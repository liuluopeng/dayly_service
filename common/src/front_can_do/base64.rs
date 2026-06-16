use base64::{Engine as _, engine::general_purpose};

/// 将字符串编码为 base64 格式
pub fn base64_encode(input: &str) -> String {
    general_purpose::STANDARD.encode(input)
}

/// 将 base64 编码的字符串解码为原始字符串
/// 如果解码失败，返回 None
pub fn base64_decode(input: &str) -> Option<String> {
    general_purpose::STANDARD
        .decode(input)
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_encode() {
        let test_str = "Hello, World!";
        let encoded = base64_encode(test_str);
        println!("编码前: {}", test_str);
        println!("编码后: {}", encoded);
        // 验证编码结果
        assert_eq!(encoded, "SGVsbG8sIFdvcmxkIQ==");
    }

    #[test]
    fn test_base64_decode() {
        let encoded_str = "SGVsbG8sIFdvcmxkIQ==";
        let decoded = base64_decode(encoded_str).unwrap();
        println!("解码前: {}", encoded_str);
        println!("解码后: {}", decoded);
        // 验证解码结果
        assert_eq!(decoded, "Hello, World!");
    }

    #[test]
    fn test_base64_decode_invalid() {
        let invalid_str = "invalid-base64";
        let decoded = base64_decode(invalid_str);
        println!("解码无效字符串: {}", invalid_str);
        println!("解码结果: {:?}", decoded);
        // 验证无效输入返回 None
        assert_eq!(decoded, None);
    }

    #[test]
    fn test_base64_roundtrip() {
        let test_strings = [
            "",
            "Hello",
            "Hello, World!",
            "This is a test string with multiple words",
            "测试中文字符串",
        ];

        for test_str in &test_strings {
            let encoded = base64_encode(test_str);
            let decoded = base64_decode(&encoded).unwrap();
            println!("原始字符串: {}", test_str);
            println!("编码后: {}", encoded);
            println!("解码后: {}", decoded);
            // 验证编码解码往返后结果一致
            assert_eq!(decoded, *test_str);
        }
    }
}
