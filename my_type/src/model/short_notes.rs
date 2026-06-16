use serde::{Deserialize, Serialize};
use uuid::Uuid;

// 为所有环境定义结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(sqlx::FromRow))]
pub struct ShortNote {
    pub id: Uuid,
    pub content: Option<String>,
    pub view_id: Option<Uuid>,
    pub view_name: Option<String>,
    pub created_at: chrono::DateTime<chrono::Local>,
    pub user_id: Option<Uuid>,
}

crate::impl_display!(self, ShortNote,
    "ID"   => self.id,
    "视图" => self.view_name.as_deref().unwrap_or("无"),
    "内容" => self.content.as_ref().map(|c| {
        if c.len() > 50 { format!("{}...", &c[..50]) } else { c.clone() }
    }).unwrap_or_else(|| "无内容".to_string())
);

impl ShortNote {
    /// 返回格式化的显示字符串
    pub fn display(&self) -> String {
        let content_preview = self
            .content
            .as_ref()
            .map(|c| {
                if c.len() > 50 {
                    format!("{}...", &c[..50])
                } else {
                    c.clone()
                }
            })
            .unwrap_or_else(|| "(无内容)".to_string());

        let view_name_display = self
            .view_name
            .as_ref()
            .map(|v| format!("[{}]", v))
            .unwrap_or_else(|| "[未命名]".to_string());

        format!(
            "{} {} - {}",
            view_name_display,
            self.created_at.format("%Y-%m-%d %H:%M"),
            content_preview
        )
    }

    /// 返回简短显示（用于列表）
    pub fn display_short(&self) -> String {
        let content_preview = self
            .content
            .as_ref()
            .map(|c| {
                if c.len() > 30 {
                    format!("{}...", &c[..30])
                } else {
                    c.clone()
                }
            })
            .unwrap_or_else(|| "(无内容)".to_string());

        format!(
            "{} - {}",
            self.id.to_string()[..8].to_string(),
            content_preview
        )
    }

    /// 返回详细显示（包含完整信息）
    pub fn display_detail(&self) -> String {
        format!(
            "ID: {}\n视图名: {}\n内容: {}\n创建时间: {}",
            self.id,
            self.view_name.as_deref().unwrap_or("(未命名)"),
            self.content.as_deref().unwrap_or("(无内容)"),
            self.created_at.format("%Y-%m-%d %H:%M:%S")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_note() -> ShortNote {
        ShortNote {
            id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            content: Some("这是一个测试笔记内容，用于测试display方法的功能".to_string()),
            view_id: Some(Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()),
            view_name: Some("测试视图".to_string()),
            created_at: chrono::Local::now(),
            user_id: None,
        }
    }

    fn create_test_note_empty() -> ShortNote {
        ShortNote {
            id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap(),
            content: None,
            view_id: Some(Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()),
            view_name: None,
            created_at: chrono::Local::now(),
            user_id: None,
        }
    }

    fn create_test_note_long_content() -> ShortNote {
        ShortNote {
            id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440002").unwrap(),
            content: Some("这是一个非常长的内容，用于测试display方法在内容超过50个字符时的截断功能，看看是否能正确显示省略号".to_string()),
            view_id: Some(Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()),
            view_name: Some("长内容测试".to_string()),
            created_at: chrono::Local::now(),
            user_id: None,
        }
    }

    #[test]
    fn test_display() {
        let note = create_test_note();
        let result = note.display();
        println!("display() 输出:\n{}\n", result);
        assert!(result.contains("[测试视图]"));
        assert!(result.contains("这是一个测试笔记内容"));
    }

    #[test]
    fn test_display_empty() {
        let note = create_test_note_empty();
        let result = note.display();
        println!("display() 空内容输出:\n{}\n", result);
        assert!(result.contains("[未命名]"));
        assert!(result.contains("(无内容)"));
    }

    #[test]
    fn test_display_short() {
        let note = create_test_note();
        let result = note.display_short();
        println!("display_short() 输出:\n{}\n", result);
        assert!(result.starts_with("550e8400"));
        assert!(result.contains("这是一个测试笔记内容"));
    }

    #[test]
    fn test_display_detail() {
        let note = create_test_note();
        let result = note.display_detail();
        println!("display_detail() 输出:\n{}\n", result);
        assert!(result.contains("ID:"));
        assert!(result.contains("视图名:"));
        assert!(result.contains("内容:"));
        assert!(result.contains("创建时间:"));
        assert!(result.contains("测试视图"));
        assert!(result.contains("这是一个测试笔记内容"));
    }

    #[test]
    fn test_display_long_content() {
        let note = create_test_note_long_content();
        let display_result = note.display();
        let short_result = note.display_short();

        println!("长内容 display() 输出:\n{}\n", display_result);
        println!("长内容 display_short() 输出:\n{}\n", short_result);

        // 验证长内容被截断并添加省略号
        assert!(display_result.contains("..."));
        assert!(short_result.contains("..."));
    }
}
