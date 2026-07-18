pub mod api {
    pub mod base;
    pub mod client;
    pub mod clipboard;
    pub mod dict;
    pub mod files;
    pub mod ggtt;
    pub mod hello;
    pub mod melatonin;
    pub mod images;
    pub mod media_paths;
    pub mod note;
    pub mod openai;
    pub mod chat;
    pub mod pinyin;
    pub mod short_note;
    pub mod songs;
    pub mod user;
    pub mod user_directories;
    pub mod videos;
    pub mod whitenoise;
}

pub mod read_metadata;
#[cfg(not(target_arch = "wasm32"))]
pub mod color_extract;
#[cfg(not(target_arch = "wasm32"))]
pub mod mhtml;
#[cfg(feature = "ocr")]
pub mod ocr;

pub mod front_can_do {
    pub mod base64;
    pub mod get_url_from_mhtml;
    pub mod image_convert;
    pub mod password;
    pub mod qrcode;
    pub mod qrscan;
    pub mod timestamp;
    pub mod uuid;
}
