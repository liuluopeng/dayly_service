use std::fmt;

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

    /// 计算字符串的视觉宽度（中文字符算2个宽度）
    fn visual_width(s: &str) -> usize {
        s.chars()
            .map(|c| {
                if c.is_ascii() {
                    1
                } else {
                    2 // 中文字符通常占2个字符宽度
                }
            })
            .sum()
    }

    /// 截断字符串到指定视觉宽度
    fn truncate_to_width(s: &str, max_width: usize) -> String {
        let mut width = 0;
        let mut result = String::new();
        
        for c in s.chars() {
            let char_width = if c.is_ascii() { 1 } else { 2 };
            if width + char_width > max_width {
                break;
            }
            result.push(c);
            width += char_width;
        }
        
        result
    }

    /// 格式化表格行
    pub fn format_row(&self, label: &str, label_color: &str, content: &str) -> String {
        let colored_label = crate::utils::color::colorize(label, label_color);
        let label_visual_width = Self::visual_width(label);
        let content_max_width = self.width - 5 - label_visual_width - 4;
        
        let content_visual_width = Self::visual_width(content);
        let truncated_content = if content_visual_width > content_max_width {
            let truncated = Self::truncate_to_width(content, content_max_width - 3);
            format!("{}...", truncated)
        } else {
            content.to_string()
        };
        
        let truncated_visual_width = Self::visual_width(&truncated_content);
        let padding = content_max_width - truncated_visual_width;
        
        format!(
            "│ {} : {}{}│",
            colored_label,
            truncated_content,
            " ".repeat(padding)
        )
    }

    /// 格式化标题行
    pub fn format_title(&self, title: &str, title_color: &str) -> String {
        let colored_title = crate::utils::color::colorize(title, title_color);
        let title_visual_width = Self::visual_width(title);
        let padding = self.width - 4 - title_visual_width;
        let left_padding = padding / 2;
        let right_padding = padding - left_padding;
        format!(
            "│ {}{}{}│",
            " ".repeat(left_padding),
            colored_title,
            " ".repeat(right_padding)
        )
    }

    /// 一句话生成完整的表格
    ///
    /// # Arguments
    /// * `f` - 格式化器
    /// * `title` - 表格标题
    /// * `title_color` - 标题颜色
    /// * `rows` - 表格行数据，每个元素是 (标签, 颜色, 内容)
    ///
    /// # Examples
    /// ```
    /// use my_type::utils::table::TableFormatter;
    /// use my_type::utils::color::{GREEN, BLUE, YELLOW};
    ///
    /// let rows = vec![
    ///     ("ID", BLUE, "123"),
    ///     ("Name", GREEN, "John"),
    ///     ("Age", YELLOW, "25"),
    /// ];
    /// let output = TableFormatter::to_string("User", GREEN, &rows);
    /// println!("{}", output);
    /// ```
    pub fn display<T: AsRef<str>>(
        f: &mut fmt::Formatter<'_>,
        title: &str,
        title_color: &str,
        rows: &[(&str, &str, T)],
    ) -> fmt::Result {
        let formatter = Self::default();

        writeln!(f)?;
        writeln!(f, "{}", formatter.top_border())?;
        writeln!(f, "{}", formatter.format_title(title, title_color))?;
        writeln!(f, "{}", formatter.separator())?;

        for (label, color, content) in rows {
            writeln!(
                f,
                "{}",
                formatter.format_row(label, color, content.as_ref())
            )?;
        }

        writeln!(f, "{}", formatter.bottom_border())?;

        Ok(())
    }

    /// 生成完整的表格字符串
    pub fn to_string<T: AsRef<str>>(
        title: &str,
        title_color: &str,
        rows: &[(&str, &str, T)],
    ) -> String {
        use std::fmt::Write;
        let mut buffer = String::new();
        let _ = writeln!(buffer, "{}", Self::default().top_border());
        let _ = writeln!(
            buffer,
            "{}",
            Self::default().format_title(title, title_color)
        );
        let _ = writeln!(buffer, "{}", Self::default().separator());
        for (label, color, content) in rows {
            let _ = writeln!(
                buffer,
                "{}",
                Self::default().format_row(label, color, content.as_ref())
            );
        }
        let _ = writeln!(buffer, "{}", Self::default().bottom_border());
        buffer
    }
}
