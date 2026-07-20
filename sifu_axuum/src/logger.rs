// 日志格式化器 — 使用 tracing 内置的颜色方案
// 不再自定义 ANSI 染色，交给 tracing_subscriber 的 with_ansi 控制
//
// 保留此模块只是为了不破坏 pub use logger::* 的导出
// 实际格式化由 main.rs 中的 tracing_subscriber::fmt() 处理

pub struct ColoredFields;
pub struct HttpFormatter;
