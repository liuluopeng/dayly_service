pub mod config {
    pub mod env;
    pub use env::{AppEnv, ServerConfig};
}
pub mod controller {
    pub mod chat;
    pub mod clipboard;
    pub mod dict;
    pub mod files;
    pub mod ggtt;
    pub mod images;
    pub mod media_paths;
    pub mod melatonin;
    pub mod mhtml_convert;
    pub mod note;
    pub mod ocr;
    pub mod openai;
    pub mod openai_session;
    pub mod short_notes;
    pub mod songs;
    pub mod user;
    pub mod user_directories;
    pub mod videos;
    pub mod webrtc;

    pub use chat::chat_routes;
    pub use clipboard::clipboard_routes;
    pub use dict::{dict_resource_routes, dict_routes};
    pub use files::file_routes;
    pub use ggtt::{ggtt_routers, search_ggtt};
    pub use images::images_routes;
    pub use media_paths::media_paths_routes;
    pub use melatonin::melatonin_routes;
    pub use mhtml_convert::mhtml_routes;
    pub use note::note_routes;
    pub use ocr::ocr_routes;
    pub use openai::openai_routes;
    pub use openai_session::openai_session_routes;
    pub use short_notes::short_notes_routes;
    pub use songs::songs_routes;
    pub use user::{secured_user_routes, user_routes};
    pub use user_directories::admin_user_dir_routes;
    pub use videos::videos_routes;
    pub use webrtc::{SignalingState, webrtc_routes};

    pub mod pinyin;
}
pub mod graphql;
pub mod grpc;
mod handlers;
pub mod logger;
pub mod middleware;
pub mod route;
pub mod utils;

pub use logger::{ColoredFields, HttpFormatter};
pub use route::create_app;

#[cfg(test)]
pub mod test_helpers;
