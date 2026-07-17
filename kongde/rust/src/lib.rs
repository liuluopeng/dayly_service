pub mod api {
    pub mod color;

    pub mod logger_bridge;
    pub mod metadata;
    // music_cache/db/runtime 需要 WASM stub（依赖 sqlx/rusqlite/tokio）
    #[cfg(not(target_arch = "wasm32"))]
    #[path = "music_cache.rs"]
    pub mod music_cache;
    #[cfg(target_arch = "wasm32")]
    #[path = "music_cache_wasm.rs"]
    pub mod music_cache;

    #[cfg(not(target_arch = "wasm32"))]
    #[path = "db.rs"]
    pub mod db;
    #[cfg(target_arch = "wasm32")]
    #[path = "db_wasm.rs"]
    pub mod db;

    #[cfg(not(target_arch = "wasm32"))]
    #[path = "runtime.rs"]
    pub mod runtime;
    #[cfg(target_arch = "wasm32")]
    #[path = "runtime_wasm.rs"]
    pub mod runtime;

    pub mod simple;

    pub mod wifi_api {
        // chat/init/webrtc 依赖 tokio/ring，WASM 用 stub
        #[cfg(not(target_arch = "wasm32"))]
        #[path = "chat.rs"]
        pub mod chat;
        #[cfg(target_arch = "wasm32")]
        #[path = "chat_wasm.rs"]
        pub mod chat;

        #[cfg(not(target_arch = "wasm32"))]
        #[path = "init.rs"]
        pub mod init;
        #[cfg(target_arch = "wasm32")]
        #[path = "init_wasm.rs"]
        pub mod init;

        #[cfg(not(target_arch = "wasm32"))]
        #[path = "webrtc.rs"]
        pub mod webrtc;
        #[cfg(target_arch = "wasm32")]
        #[path = "webrtc_wasm.rs"]
        pub mod webrtc;

        // 仅用 reqwest（已支持 WASM），无需 stub
        pub mod clipboard;
        pub mod dict;
        pub mod files;
        pub mod ggtt;
        pub mod melatonin;
        pub mod note;
        pub mod song;
        pub mod user;
    }

    pub mod utils {
        pub mod base64;
        pub mod calculator;
        pub mod gif_decode;
        pub mod password;
        pub mod timestamp;
        pub mod uuid;
    }
}
mod frb_generated;

pub mod model {
    pub mod music;
    pub mod todo;
}
