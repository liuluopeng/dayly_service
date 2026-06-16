use infer;
use serde::{Deserialize, Serialize};
use std::fmt;

// 仅在非 WebAssembly 环境中导入 sqlx
#[cfg(not(target_arch = "wasm32"))]
use sqlx::FromRow;

// 导入颜色工具
use crate::utils::color::{BLUE, CYAN, GREEN, MAGENTA, YELLOW};
use crate::utils::table::TableFormatter;

// 为所有环境定义结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(FromRow))]
pub struct Note {
    pub id: uuid::Uuid,
    pub text: Option<String>,
    pub simple_text: Option<String>,
    pub raw_content: Option<Vec<u8>>,
    pub filepath: Option<String>,
    pub sha256: Option<String>,
    pub filename: Option<String>,
    pub file_info: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Local>,
    pub updated_at: chrono::DateTime<chrono::Local>,
}

// 实现 Display trait
impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let filename = self.filename.as_deref().unwrap_or("N/A");
        let text_preview = self
            .text
            .as_deref()
            .unwrap_or("N/A")
            .chars()
            .take(50)
            .collect::<String>();
        let text_preview = if text_preview.len() >= 50 {
            format!("{}...", text_preview)
        } else {
            text_preview
        };

        // 分析 raw_content
        let raw_content_info = if let Some(ref content) = self.raw_content {
            let content_type = if let Some(kind) = infer::get(content) {
                format!("{} ({})", kind.mime_type(), kind.extension())
            } else if content
                .iter()
                .all(|&b| b.is_ascii() && (b.is_ascii_graphic() || b.is_ascii_whitespace()))
            {
                "ASCII text".to_string()
            } else if String::from_utf8(content.to_vec()).is_ok() {
                "UTF-8 text".to_string()
            } else if content.len() > 0 {
                "data".to_string()
            } else {
                "empty".to_string()
            };
            format!("{} ({} bytes)", content_type, content.len())
        } else {
            "N/A".to_string()
        };

        let id_str = self.id.to_string();
        let created_str = self.created_at.format("%Y-%m-%d %H:%M:%S").to_string();
        let updated_str = self.updated_at.format("%Y-%m-%d %H:%M:%S").to_string();

        let rows = vec![
            ("ID", BLUE, id_str.as_str()),
            ("Filename", GREEN, filename),
            ("Text", YELLOW, &text_preview),
            ("Raw Content", MAGENTA, &raw_content_info),
            ("Created", CYAN, created_str.as_str()),
            ("Updated", CYAN, updated_str.as_str()),
        ];

        TableFormatter::display(f, "Note", GREEN, &rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;
    use strip_ansi_escapes::strip;
    use uuid::Uuid;

    #[test]
    fn test_note_display() {
        // 创建一个测试用的 Note 实例
        let note = Note {
            id: Uuid::nil(),
            text: Some("这是一个测试笔记，包含一些文本内容。".to_string()),
            simple_text: Some("测试笔记".to_string()),
            raw_content: Some(b"Hello, World!".to_vec()),
            filepath: Some("/path/to/note.txt".to_string()),
            sha256: Some("test_sha256".to_string()),
            filename: Some("note.txt".to_string()),
            file_info: None,
            created_at: Local::now(),
            updated_at: Local::now(),
        };

        // 测试 Display 实现
        let display_output = format!("{}", note);
        println!("Display output: {}", display_output);

        // 移除 ANSI 转义序列
        let stripped_output = String::from_utf8(strip(display_output.as_bytes())).unwrap();

        // 验证输出包含预期的信息
        assert!(stripped_output.contains("note.txt"));
        assert!(stripped_output.contains("这是一个测试笔记，包含一些文本内容。"));
        assert!(stripped_output.contains("ASCII text"));
        assert!(stripped_output.contains("13 bytes"));
        assert!(stripped_output.contains("Raw Content"));
    }

    #[test]
    fn test_note_display_with_empty_raw_content() {
        // 创建一个 raw_content 为空的 Note 实例
        let note = Note {
            id: Uuid::nil(),
            text: Some("测试笔记".to_string()),
            simple_text: Some("测试笔记".to_string()),
            raw_content: None,
            filepath: Some("/path/to/note.txt".to_string()),
            sha256: Some("test_sha256".to_string()),
            filename: Some("note.txt".to_string()),
            file_info: None,
            created_at: Local::now(),
            updated_at: Local::now(),
        };

        // 测试 Display 实现
        let display_output = format!("{}", note);
        println!("Display output (empty raw_content): {}", display_output);

        // 移除 ANSI 转义序列
        let stripped_output = String::from_utf8(strip(display_output.as_bytes())).unwrap();

        // 验证输出包含预期的信息
        assert!(stripped_output.contains("note.txt"));
        assert!(stripped_output.contains("测试笔记"));
        assert!(stripped_output.contains("Raw Content : N/A"));
    }
}
