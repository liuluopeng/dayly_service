#![allow(missing_docs)] // 除 api 模块外，DTO/工具模块字段名自解释

//! # common —— 跨平台共享库
//!
//! 被服务端（sifu_axuum）、Flutter 核心（kongde/rust）、WASM 前端
//! （webbvueetauri/src-wasm）与本地工具（local-agent）共同使用的
//! 基础能力：HTTP API 封装、OCR、MHTML 解析、小工具算法等。

/// HTTP API 客户端与各接口的封装（native 与 wasm 双平台）
pub mod api {
    #![warn(missing_docs)]
    /// 统一错误/响应类型（ApiError、ApiResponse 等）
    pub mod base;
    /// 聊天消息 API
    pub mod chat;
    /// 统一 HTTP 客户端（token、URL、请求封装）
    pub mod client;
    /// 剪贴板历史 API
    pub mod clipboard;
    /// 词典查询 API（柯林斯/朗文/现代汉语）
    pub mod dict;
    /// 文件浏览/读取 API
    pub mod files;
    /// GGTT 码查询 API
    pub mod ggtt;
    /// 示例接口
    pub mod hello;
    /// 图片库 API
    pub mod images;
    /// 媒体路径管理 API
    pub mod media_paths;
    /// Melatonin 影视库 API
    pub mod melatonin;
    /// 笔记 API
    pub mod note;
    /// OpenAI 会话 API
    pub mod openai;
    /// 拼音查询 API
    pub mod pinyin;
    /// 短笔记 API
    pub mod short_note;
    /// 歌曲 API
    pub mod songs;
    /// 用户登录/登出 API
    pub mod user;
    /// 用户授权目录管理 API
    pub mod user_directories;
    /// 视频库 API
    pub mod videos;
    /// 白噪音资源 API
    pub mod whitenoise;
}

/// 图片主色提取（非 wasm 目标）
#[cfg(not(target_arch = "wasm32"))]
pub mod color_extract;
/// MHTML 解析与转 Markdown（非 wasm 目标）
#[cfg(not(target_arch = "wasm32"))]
pub mod mhtml;
/// OCR 文字识别（det + rec 双模型，feature: ocr）
#[cfg(feature = "ocr")]
pub mod ocr;
/// 读取音频文件元数据（标题/艺术家/封面等）
pub mod read_metadata;

/// 前端可用的"工具函数"集合（游戏、二维码、编码等）
pub mod front_can_do {
    /// Base64 编解码
    pub mod base64;
    /// 2048 游戏
    pub mod game2048;
    /// 从 MHTML 提取 URL 主题
    pub mod get_url_from_mhtml;
    /// 图片格式检测/转换
    pub mod image_convert;
    /// 扫雷游戏
    pub mod minesweeper;
    /// 密码生成
    pub mod password;
    /// QR 码生成
    pub mod qrcode;
    /// QR 码识别
    pub mod qrscan;
    /// 贪吃蛇游戏
    pub mod snake;
    /// 俄罗斯方块游戏
    pub mod tetris;
    /// 时间戳工具
    pub mod timestamp;
    /// UUID 生成与校验
    pub mod uuid;
}
