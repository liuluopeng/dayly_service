pub fn get_url_from_mhtml(mhtml_content: &str) -> Option<String> {
    // 尝试从Snapshot-Content-Location提取URL
    for line in mhtml_content.lines() {
        let line = line.trim();
        if line.starts_with("Snapshot-Content-Location:") {
            if let Some(url) = line.strip_prefix("Snapshot-Content-Location:") {
                let url = url.trim();
                if !url.is_empty() {
                    return Some(url.to_string());
                }
            }
        }
    }

    // 尝试从Content-Location提取URL
    for line in mhtml_content.lines() {
        let line = line.trim();
        if line.starts_with("Content-Location:") {
            if let Some(url) = line.strip_prefix("Content-Location:") {
                let url = url.trim();
                if !url.is_empty() && url.starts_with("http") {
                    return Some(url.to_string());
                }
            }
        }
    }

    // 尝试从From字段提取URL（某些MHTML格式）
    for line in mhtml_content.lines() {
        let line = line.trim();
        if line.starts_with("From:") {
            if let Some(url_part) = line.strip_prefix("From:") {
                let url_part = url_part.trim();
                // 处理 "From: <Saved by Blink>" 格式
                if url_part.contains("http") {
                    if let Some(start) = url_part.find("http") {
                        return Some(url_part[start..].to_string());
                    }
                }
            }
        }
    }

    None
}

pub fn get_subject_from_mhtml(mhtml_content: &str) -> Option<String> {
    let lines: Vec<&str> = mhtml_content.lines().collect();
    let mut subject_lines: Vec<String> = Vec::new();
    let mut in_subject = false;

    for line in lines {
        let trimmed = line.trim();

        if trimmed.starts_with("Subject:") {
            in_subject = true;
            if let Some(subject) = trimmed.strip_prefix("Subject:") {
                let subject = subject.trim();
                if !subject.is_empty() {
                    subject_lines.push(subject.to_string());
                }
            }
        } else if in_subject {
            // 检查是否是续行（以空格开头且包含编码标记）
            if line.starts_with(' ') && (trimmed.starts_with("=?") || trimmed.contains("?=")) {
                subject_lines.push(trimmed.to_string());
            } else {
                // Subject结束
                break;
            }
        }
    }

    if subject_lines.is_empty() {
        return None;
    }

    // 分别解码每一行，然后合并
    let mut full_subject = String::new();
    for line in subject_lines {
        let decoded = decode_rfc2047(&line);
        full_subject.push_str(&decoded);
    }

    Some(full_subject.trim().to_string())
}

/// 解码RFC 2047编码的字符串
fn decode_rfc2047(encoded: &str) -> String {
    // 处理多个编码段，如：=?utf-8?Q?xxx?= =?utf-8?Q?yyy?=
    let mut result = String::new();
    let mut chars = encoded.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '=' && chars.peek() == Some(&'?') {
            // 跳过 =?
            chars.next();

            // 读取charset
            let mut charset = String::new();
            while let Some(c) = chars.next() {
                if c == '?' {
                    break;
                }
                charset.push(c);
            }

            // 读取encoding (Q或B)
            let encoding = chars.next();

            // 跳过 ?
            if chars.peek() == Some(&'?') {
                chars.next();
            }

            // 读取encoded-text直到 ?=
            let mut encoded_text = String::new();
            while let Some(c) = chars.next() {
                if c == '?' {
                    if chars.peek() == Some(&'=') {
                        chars.next(); // 跳过 =
                        break;
                    }
                }
                encoded_text.push(c);
            }

            // 解码
            if let Some(enc) = encoding {
                let decoded = match enc {
                    'Q' | 'q' => decode_quoted_printable(&encoded_text),
                    'B' | 'b' => decode_base64(&encoded_text),
                    _ => encoded_text,
                };
                result.push_str(&decoded);
            }
        } else if c == ' ' {
            // 跳过编码段之间的空格
            continue;
        } else {
            result.push(c);
        }
    }

    // 如果没有解码成功，返回原始字符串
    if result.is_empty() {
        encoded.to_string()
    } else {
        result
    }
}

/// 解码Quoted-Printable编码
fn decode_quoted_printable(encoded: &str) -> String {
    let mut result_bytes: Vec<u8> = Vec::new();
    let mut chars = encoded.chars();

    while let Some(c) = chars.next() {
        if c == '=' {
            // 读取下一个两个字符作为hex值
            let hex1 = chars.next();
            let hex2 = chars.next();

            if let (Some(h1), Some(h2)) = (hex1, hex2) {
                let hex_str = format!("{}{}", h1, h2);
                if let Ok(byte) = u8::from_str_radix(&hex_str, 16) {
                    result_bytes.push(byte);
                } else {
                    result_bytes.push(b'=');
                    result_bytes.push(h1 as u8);
                    result_bytes.push(h2 as u8);
                }
            } else {
                result_bytes.push(b'=');
                if let Some(h1) = hex1 {
                    result_bytes.push(h1 as u8);
                }
            }
        } else if c == '_' {
            // RFC 2047中，空格编码为下划线
            result_bytes.push(b' ');
        } else {
            result_bytes.push(c as u8);
        }
    }

    // 将字节数组转换为UTF-8字符串
    String::from_utf8_lossy(&result_bytes).to_string()
}

/// 解码Base64编码
fn decode_base64(encoded: &str) -> String {
    use base64::{Engine as _, engine::general_purpose};

    // 移除空白字符
    let cleaned: String = encoded.chars().filter(|c| !c.is_whitespace()).collect();

    match general_purpose::STANDARD.decode(&cleaned) {
        Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
        Err(_) => encoded.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_url_from_mhtml() {
        let mhtml_content = r#"
From: <Saved by Blink>
Snapshot-Content-Location: https://weibo.com/7968715266/QjQjd1uqF?pagetype=profilefeed
Subject: =?utf-8?Q?Some=20Chine...=20-=20@=E6=B3=B0=E5=8B=92=E8=A7=86=E8=A7=92=E7?=
 =?utf-8?Q?=9A=84=E5=BE=AE=E5=8D=9A=20-=20=E5=BE=AE=E5=8D=9A?=
Date: Mon, 29 Dec 2025 14:30:42 +0800
MIME-Version: 1.0
Content-Type: multipart/related;
	type="text/html";
	boundary="----MultipartBoundary--QrDo8AIIq2MZfquBx9U4OkrkLeBLFfAHinl4jiWrbm----"

        "#;
        let url = get_url_from_mhtml(mhtml_content).unwrap();
        println!("URL: {}", url);
        assert_eq!(
            url,
            "https://weibo.com/7968715266/QjQjd1uqF?pagetype=profilefeed"
        );
    }

    #[test]
    fn test_get_url_from_content_location() {
        let mhtml_content = r#"
MIME-Version: 1.0
Content-Type: multipart/related;
Content-Location: https://www.example.com/page.html
Subject: Test Page

        "#;
        let url = get_url_from_mhtml(mhtml_content).unwrap();
        println!("URL: {}", url);
        assert_eq!(url, "https://www.example.com/page.html");
    }

    #[test]
    fn test_get_url_not_found() {
        let mhtml_content = r#"
MIME-Version: 1.0
Content-Type: text/html
Subject: Test Page

        "#;
        let url = get_url_from_mhtml(mhtml_content);
        assert!(url.is_none());
    }

    #[test]
    fn test_get_subject_from_mhtml() {
        let mhtml_content = r#"
From: <Saved by Blink>
Snapshot-Content-Location: https://www.zhihu.com/p/123
Subject: =?utf-8?Q?2013=E5=B9=B4=EF=BC=8C=E4=B8=AD=E5=9B=BD=E5=90=91=E7=BE=8E=E5=9B=BD=E6=89=BF=E8=AF=BA=E6=94=B9=E5=96=84=E4%BA=BA=E6=B0=91=E7=94=9F=E6=B4=BB=E8=B4=A8=E9=87=8F=E3=80=82=20-=20=E7=9F=A5=E4=B9=8E?=
Date: Sun, 23 Nov 2025 12:21:58 +0800
MIME-Version: 1.0

        "#;
        let subject = get_subject_from_mhtml(mhtml_content).unwrap();
        println!("Subject: {}", subject);
        // 检查解码后的内容包含关键信息
        assert!(subject.contains("2013"));
        assert!(subject.contains("年"));
    }

    #[test]
    fn test_get_subject_multiline() {
        // 测试多行Subject
        let mhtml_content = r#"
From: <Saved by Blink>
Snapshot-Content-Location: https://weibo.com/123
Subject: =?utf-8?Q?=E6=96=AF=E5=A5=8E=E5=A5=87=E5=A4=A7=E7=8E=8B=EF=BC=88=E7=89=A2?=
 =?utf-8?Q?A=EF=BC=89=E7=9A=84=E8=A7=86=E9=A2=91=E5=86=85=E5=AE=B9=E7=BC?=
 =?utf-8?Q?=BA=E4=B9=8F=E7=9B=B4=E6=8E=A5=E8=AF=81=E6=8D=AE=EF=BC=8C=E4=B8?=
Date: Tue, 23 Dec 2025 23:10:15 +0800
MIME-Version: 1.0

        "#;
        let subject = get_subject_from_mhtml(mhtml_content).unwrap();
        println!("Subject: {}", subject);
        // 应该包含完整的中文
        assert!(subject.contains("斯奎奇"));
    }

    #[test]
    fn test_get_subject_multiline_with_chinese_punctuation() {
        // 测试包含中文标点的多行Subject
        let mhtml_content = r#"
From: <Saved by Blink>
Snapshot-Content-Location: https://www.zhihu.com/question/123
Subject: =?utf-8?Q?TikTok=E7=BE=8E=E5=9B=BD=E6=96=B9=E6=A1=88=E8=90=BD=E5=9C=B0?=
 =?utf-8?Q?=EF=BC=8C=E7=BB=93=E6=9D=9F=E9=95=BF=E8=BE=BE=E5=85=AD?=
Date: Mon, 1 Jan 2026 10:00:00 +0800
MIME-Version: 1.0

        "#;
        let subject = get_subject_from_mhtml(mhtml_content).unwrap();
        println!("Subject: {}", subject);
        // 应该包含中文标点和完整文本
        assert!(subject.contains("TikTok"));
        assert!(subject.contains("美国方案落地"));
        assert!(subject.contains("，")); // 中文逗号
        assert!(subject.contains("结束长达六"));
    }

    #[test]
    fn test_get_subject_plain() {
        let mhtml_content = r#"
MIME-Version: 1.0
Subject: Simple Test Page
Content-Type: text/html

        "#;
        let subject = get_subject_from_mhtml(mhtml_content).unwrap();
        println!("Subject: {}", subject);
        assert_eq!(subject, "Simple Test Page");
    }

    #[test]
    fn test_get_subject_not_found() {
        let mhtml_content = r#"
MIME-Version: 1.0
Content-Type: text/html

        "#;
        let subject = get_subject_from_mhtml(mhtml_content);
        assert!(subject.is_none());
    }

    #[test]
    fn test_decode_rfc2047() {
        // 测试解码RFC 2047编码
        let encoded = "=?utf-8?Q?Hello=20World?=";
        let decoded = decode_rfc2047(encoded);
        assert_eq!(decoded, "Hello World");
    }

    #[test]
    fn test_decode_chinese() {
        // 测试解码中文
        let encoded = "=?utf-8?Q?=E4=B8=AD=E6=96=87=E6=B5=8B=E8=AF=95?=";
        let decoded = decode_rfc2047(encoded);
        println!("Decoded: {}", decoded);
        assert_eq!(decoded, "中文测试");
    }

    #[test]
    fn test_decode_chinese_punctuation() {
        // 测试解码中文标点
        let encoded = "=?utf-8?Q?=EF=BC=8C=E3=80=82?="; // ，。
        let decoded = decode_rfc2047(encoded);
        println!("Decoded punctuation: {}", decoded);
        assert!(decoded.contains("，"));
        assert!(decoded.contains("。"));
    }
}
