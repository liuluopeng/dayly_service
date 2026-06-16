// 颜色常量和格式化工具

/// 重置颜色的 ANSI 转义序列
pub const RESET: &str = "\x1b[0m";

/// 绿色的 ANSI 转义序列
pub const GREEN: &str = "\x1b[32m";

/// 青色的 ANSI 转义序列
pub const CYAN: &str = "\x1b[36m";

/// 黄色的 ANSI 转义序列
pub const YELLOW: &str = "\x1b[33m";

/// 蓝色的 ANSI 转义序列
pub const BLUE: &str = "\x1b[34m";

/// 洋红色的 ANSI 转义序列
pub const MAGENTA: &str = "\x1b[35m";

/// 红色的 ANSI 转义序列
pub const RED: &str = "\x1b[31m";

/// 用颜色包裹文本
///
/// # Examples
/// ```
/// use my_type::utils::color::{GREEN, colorize};
///
/// let colored_text = colorize("Hello", GREEN);
/// println!("{}", colored_text);
/// ```
pub fn colorize(text: &str, color: &str) -> String {
    format!("{}{}{}", color, text, RESET)
}

/// 表格格式化器
pub struct TableFormatter {
    width: usize,
}

impl TableFormatter {
    /// 创建一个新的表格格式化器
    pub fn new(width: usize) -> Self {
        TableFormatter { width }
    }

    /// 创建默认宽度的表格格式化器（宽度为 60）
    pub fn default() -> Self {
        TableFormatter { width: 60 }
    }

    /// 生成表格顶边框
    pub fn top_border(&self) -> String {
        format!("┌{}┐", "─".repeat(self.width - 2))
    }

    /// 生成表格底边框
    pub fn bottom_border(&self) -> String {
        format!("└{}┘", "─".repeat(self.width - 2))
    }

    /// 生成表格分隔线
    pub fn separator(&self) -> String {
        format!("├{}┤", "─".repeat(self.width - 2))
    }

    /// 格式化表格行
    ///
    /// # Arguments
    /// * `label` - 字段标签
    /// * `label_color` - 标签颜色
    /// * `content` - 字段内容
    ///
    /// # Examples
    /// ```
    /// use my_type::utils::color::{TableFormatter, BLUE, colorize};
    ///
    /// let formatter = TableFormatter::default();
    /// let row = formatter.format_row("ID", BLUE, "123");
    /// println!("{}", row);
    /// ```
    pub fn format_row(&self, label: &str, label_color: &str, content: &str) -> String {
        let colored_label = colorize(label, label_color);
        let content_width = self.width - 5 - label.len() - 4;
        let truncated_content = if content.chars().count() > content_width {
            let chars: Vec<char> = content.chars().take(content_width - 3).collect();
            format!("{}...", chars.into_iter().collect::<String>())
        } else {
            content.to_string()
        };
        format!(
            "│ {} : {}{}│",
            colored_label,
            truncated_content,
            " ".repeat(content_width - truncated_content.chars().count())
        )
    }

    /// 格式化标题行
    pub fn format_title(&self, title: &str, title_color: &str) -> String {
        let colored_title = colorize(title, title_color);
        let padding = self.width - 4 - title.len();
        let left_padding = padding / 2;
        let right_padding = padding - left_padding;
        format!(
            "│ {}{}{}│",
            " ".repeat(left_padding),
            colored_title,
            " ".repeat(right_padding)
        )
    }
}
