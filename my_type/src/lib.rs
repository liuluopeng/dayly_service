pub mod model {
    pub mod admin;
    pub mod basic_model;
    pub mod dict;
    pub mod ggtt;
    pub mod haiyu_dict;
    pub mod melatonin_movie;
    pub mod media_paths;
    pub mod single_char_pinyin;
    pub mod songs;
    pub mod images;
    pub mod user_directories;
    pub mod videos;
    pub mod short_notes;
    pub mod view_name;
    pub mod openai;
    pub mod note;
    pub mod chat;
}

pub mod dto;

pub mod utils;

/// 为结构体自动生成 ANSI 彩色 Display 实现。
///
/// 用法:
/// ```
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

