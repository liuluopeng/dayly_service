#![allow(missing_docs)] // 数据模型/DTO 字段名自解释

//! # my_type —— 共享数据模型与 DTO
//!
//! 定义跨 crate 使用的数据库模型（ORM 结构体）与 API 传输对象（DTO），
//! 以及终端格式化的工具函数。

/// 数据库模型（对应 Postgres 表结构）
pub mod model {
    /// 管理员模型
    pub mod admin;
    /// 基础模型（通用字段）
    pub mod basic_model;
    /// 聊天消息模型
    pub mod chat;
    /// 词典查询模型（词条/历史）
    pub mod dict;
    /// GGTT 模型
    pub mod ggtt;
    /// 海语词典模型
    pub mod haiyu_dict;
    /// 图片模型
    pub mod images;
    /// 媒体路径模型
    pub mod media_paths;
    /// Melatonin 电影模型
    pub mod melatonin_movie;
    /// 笔记模型
    pub mod note;
    /// OpenAI 会话模型
    pub mod openai;
    /// 短笔记模型
    pub mod short_notes;
    /// 单字拼音模型
    pub mod single_char_pinyin;
    /// 歌曲模型
    pub mod songs;
    /// 用户授权目录模型
    pub mod user_directories;
    /// 视频模型
    pub mod videos;
    /// 视图名模型
    pub mod view_name;
}

/// API 传输对象（请求/响应 DTO）
pub mod dto;

/// 工具函数（颜色/表格格式化）
pub mod utils;

/// 为结构体自动生成 ANSI 彩色 Display 实现。
///
/// 用法:
/// ```
/// use my_type::impl_display;
///
/// struct MyStruct { id: String, title: String, artist: Option<String> }
///
/// impl_display!(self, MyStruct,
///     "ID"     => self.id,
///     "标题"   => self.title,
///     "艺术家" => self.artist.as_deref().unwrap_or("无"),
/// );
/// ```
///
/// 第一个参数必须是 `self`（用于宏卫生），支持行尾逗号。每个字段的 label 颜色从调色板中轮转选取。
#[macro_export]
macro_rules! impl_display {
    ($self:ident, $struct:ty, $($label:expr => $value:expr),+ $(,)?) => {
        impl std::fmt::Display for $struct {
            fn fmt(&$self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                const RESET: &str = "\x1b[0m";
                const COLORS: [&str; 7] = [
                    "\x1b[36m", // cyan
                    "\x1b[32m", // green
                    "\x1b[33m", // yellow
                    "\x1b[35m", // magenta
                    "\x1b[34m", // blue
                    "\x1b[31m", // red
                    "\x1b[37m", // white
                ];
                let mut _idx = 0usize;
                $(
                    {
                        let color = COLORS[_idx % COLORS.len()];
                        if _idx > 0 {
                            write!(f, "\n")?;
                        }
                        write!(f, "{}{}:{}{}", color, $label, RESET, $value)?;
                        _idx += 1;
                    }
                )+
                Ok(())
            }
        }
    };
}
