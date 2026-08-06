mod api {
    pub mod admin;
    pub mod chat;
    pub mod clipboard;
    pub mod dict;
    pub mod files;
    pub mod ggtt;
    pub mod helpers;
    pub mod image;
    pub mod init;
    pub mod media_path;
    pub mod note;
    pub mod openai;
    pub mod short_note;
    pub mod song;
    pub mod user;
    pub mod video;
    pub mod ws;
    pub mod zici;
}

mod utils {
    pub mod base64;
    pub mod calculator;
    pub mod console_log;
    pub mod game2048;
    pub mod greeter;
    pub mod image_convert;
    pub mod minesweeper;
    pub mod noise;
    pub mod noise_live;
    pub mod password;
    pub mod qrcode;
    pub mod qrscan;
    pub mod snake;
    pub mod synth;
    pub mod synth_analyze;
    pub mod tetris;
    pub mod timestamp;
    pub mod uuid;
}

// 重新导出所有函数和宏
pub use api::admin::{
    add_user_directory_wasm, delete_user_directory_wasm, list_user_directories_wasm,
    list_users_wasm,
};
pub use api::chat::{
    get_contacts_wasm, get_messages_wasm, get_recent_contacts_wasm, send_message_wasm,
};
pub use api::files::{get_file_info_wasm, get_file_url_wasm, list_files_wasm};
pub use api::ggtt::search_ggtt;
pub use api::image::{get_image_folders_wasm, list_images_wasm, scan_images_wasm};
pub use api::media_path::{add_media_path_wasm, delete_media_path_wasm, list_media_paths_wasm};
pub use api::note::{
    create_note_wasm, get_note_wasm, list_notes_wasm, save_note_wasm, search_notes_wasm,
};
pub use api::openai::{
    add_message, chat_completion, create_session, delete_session, get_session,
    get_session_messages, list_sessions,
};
pub use api::short_note::{
    create_short_note_wasm, delete_short_note_wasm, get_short_note_wasm, list_short_notes_wasm,
    update_short_note_wasm,
};
pub use api::song::{
    get_all_songs_wasm, get_song_cover_wasm, get_song_file_wasm, get_song_lyrics_wasm,
    get_song_ttml_wasm, get_songs_by_album_wasm, get_songs_by_artist_wasm, get_wasm_memory,
    load_song_audio_zc, scan_songs_wasm,
};
pub use api::user::login_wasm;
pub use api::video::{list_videos_wasm, scan_videos_wasm};
pub use utils::base64::{base64_decode_wasm, base64_encode_wasm};
pub use utils::calculator::{add, add22, multiply};
pub use utils::console_log;
pub use utils::greeter::greet;
pub use utils::password::{generate_password, generate_strong_password};
pub use utils::qrcode::{generate_qr_png_wasm, generate_qr_unicode_wasm, qr_info_wasm};
pub use utils::qrscan::scan_qr_from_image_wasm;
pub use utils::timestamp::{
    get_current_local_time, get_current_timestamp, get_current_utc_time, local_to_timestamp,
    timestamp_to_local, timestamp_to_utc,
};
pub use utils::uuid::{
    generate_uuid_v4, generate_uuid_v5, generate_uuid_v6, generate_uuid_v7, validate_uuid,
};

pub use utils::game2048::*;
pub use utils::minesweeper::*;
pub use utils::snake::*;
pub use utils::tetris::*;

pub use api::clipboard::get_clipboard_history_wasm;
pub use api::init::get_base_url_wasm;
pub use api::init::init_api;
pub use api::init::set_api_port;
pub use api::zici::{
    get_direction, get_new_chars, get_new_words, get_word_frequency_api, my_console_log,
};
pub use utils::image_convert::{
    convert_image_wasm, convert_image_with_size_wasm, crop_image_wasm, detect_image_format,
    resize_image_wasm,
};
pub use utils::noise::{synth_brown_noise, synth_pink_noise, synth_rain_noise, synth_white_noise};
pub use utils::synth::{synth_bell_note, synth_piano_bright, synth_piano_note};
pub use utils::synth_analyze::{analyze_piano_samples, synth_analyzed_note};
