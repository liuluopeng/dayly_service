use regex::Regex;

fn process_sound_links(record: String, prefix_resource: &str) -> String {
    let regex_href_schema_sound =
        Regex::new(r#"([ "]href=["'])(sound://)([^"']+?)(["'])"#).unwrap();

    regex_href_schema_sound
        .replace_all(&record, |caps: &regex::Captures| {
            let prefix = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let protocol = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            let value = caps.get(3).map(|m| m.as_str()).unwrap_or("");
            let quote = caps.get(4).map(|m| m.as_str()).unwrap_or("\"");
            format!(
                r#" data-sound-url="{}/{}" {}{}{}{}"#,
                prefix_resource, value, prefix, protocol, value, quote
            )
        })
        .to_string()
}

#[test]
fn test_process_sound_links() {
    let test_cases = vec![
        r#"<a href="sound://audio.mp3">Play</a>"#,
        r#"<a href="sound://audio.mp3#t=0.5">Play</a>"#,
        r#" <a href="sound://test.mp3">"#,
        r#"<a href='sound://audio.mp3'>Play</a>"#,
        r#"<a href="entry://another_word">Link</a>"#,
        r#"<img src="image.png">"#,
        r#"<a href="style.css">CSS</a>"#,
    ];

    let prefix_resource = "http://192.168.31.58:23000/api/dict/resource/";

    for (i, test_html) in test_cases.iter().enumerate() {
        println!("测试用例 {}: {}", i + 1, test_html);

        let result = process_sound_links(test_html.to_string(), prefix_resource);
        println!("  处理后: {}", result);

        if test_html.contains("sound://") {
            assert!(
                result.contains("data-sound-url="),
                "测试用例 {} 应该包含 data-sound-url",
                i + 1
            );
            assert!(
                result.contains("href=\"sound://") || result.contains("href='sound://"),
                "测试用例 {} 应该保留原始 sound:// href",
                i + 1
            );
        } else {
            assert!(
                !result.contains("data-sound-url="),
                "测试用例 {} 不应该包含 data-sound-url",
                i + 1
            );
        }

        println!();
    }
}
