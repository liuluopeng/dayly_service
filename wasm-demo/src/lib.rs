#![allow(deprecated)]
use common::api::{
    client::ApiClient,
    clipboard::get_clipboard_history,
    dict,
    ggtt::{search_ggtt_code, SearchRequest},
    note::list_notes,
    short_note::{create_short_note, list_short_notes, CreateShortNoteRequest},
    songs::{get_all_songs, scan_songs},
    user::user_login,
};
use common::front_can_do::{
    base64::{base64_decode, base64_encode},
    game2048::Game2048,
    minesweeper::Minesweeper,
    noise::{synth_brown_noise, synth_pink_noise, synth_rain_noise, synth_white_noise},
    password::{generate_password, generate_strong_password},
    qrcode::{generate_qr_unicode, qr_info},
    snake::Snake,
    tetris::Tetris,
    timestamp, uuid,
};
use my_type::dto::SongWithUrl;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{
    Document, HtmlElement, HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement, Storage,
};

const TOKEN_KEY: &str = "wasm_demo_token";

thread_local! {
    static CLIENT: RefCell<Option<ApiClient>> = const { RefCell::new(None) };
    static SONG_LIST: RefCell<Vec<SongWithUrl>> = const { RefCell::new(Vec::new()) };
    static SONG_INDEX: RefCell<usize> = const { RefCell::new(0) };
}

fn log(msg: &str) {
    web_sys::console::log_1(&msg.into());
}

fn storage() -> Option<Storage> {
    let w = web_sys::window()?;
    w.local_storage().ok()?
}

fn save_token(token: &str) {
    if let Some(s) = storage() {
        s.set_item(TOKEN_KEY, token).ok();
    }
    set_token_inner(token);
}

fn load_token() -> Option<String> {
    let s = storage()?;
    let token = s.get_item(TOKEN_KEY).ok()??;
    if token.is_empty() {
        None
    } else {
        Some(token)
    }
}

fn clear_token() {
    if let Some(s) = storage() {
        s.remove_item(TOKEN_KEY).ok();
    }
    set_token_inner("");
}

fn doc() -> Document {
    web_sys::window().unwrap().document().unwrap()
}

fn el(id: &str) -> HtmlElement {
    doc()
        .get_element_by_id(id)
        .unwrap()
        .dyn_into::<HtmlElement>()
        .unwrap()
}

fn input(id: &str) -> String {
    doc()
        .get_element_by_id(id)
        .unwrap()
        .dyn_into::<HtmlInputElement>()
        .unwrap()
        .value()
}

fn select_val(id: &str) -> String {
    doc()
        .get_element_by_id(id)
        .unwrap()
        .dyn_into::<HtmlSelectElement>()
        .unwrap()
        .value()
}

fn textarea(id: &str) -> String {
    doc()
        .get_element_by_id(id)
        .unwrap()
        .dyn_into::<HtmlTextAreaElement>()
        .unwrap()
        .value()
}

fn set_html(id: &str, html: &str) {
    doc().get_element_by_id(id).unwrap().set_inner_html(html);
}
fn set_text(id: &str, text: &str) {
    doc()
        .get_element_by_id(id)
        .unwrap()
        .set_text_content(Some(text));
}
fn hide(id: &str) {
    el(id).set_attribute("style", "display:none").ok();
}
fn unhide(id: &str) {
    el(id).set_attribute("style", "display:block").ok();
}

fn disable_btn(id: &str, disabled: bool) {
    if disabled {
        el(id).set_attribute("disabled", "true").ok();
    } else {
        el(id).remove_attribute("disabled").ok();
    }
}

fn on_keydown(id: &str, key: &str, mut f: impl FnMut() + 'static) {
    let cb = Closure::wrap(Box::new(move |e: web_sys::KeyboardEvent| {
        if e.key() == key {
            f();
        }
    }) as Box<dyn FnMut(_)>);
    doc()
        .get_element_by_id(id)
        .unwrap()
        .add_event_listener_with_callback("keydown", cb.as_ref().dyn_ref().unwrap())
        .ok();
    cb.forget();
}

fn on_click(id: &str, f: impl FnMut() + 'static) {
    let cb = Closure::wrap(Box::new(f) as Box<dyn FnMut()>);
    doc()
        .get_element_by_id(id)
        .unwrap()
        .add_event_listener_with_callback("click", cb.as_ref().dyn_ref().unwrap())
        .ok();
    cb.forget();
}

// ==================== NAVIGATION ====================

const ALL_PAGES: &[&str] = &[
    "page-login",
    "page-ggtt",
    "page-dict",
    "page-qrcode",
    "page-password",
    "page-uuid",
    "page-base64",
    "page-timestamp",
    "page-clipboard",
    "page-notes",
    "page-zici-chars",
    "page-zici-words",
    "page-search-history",
    "page-short-notes",
    "page-songs",
    "page-videos",
    "page-g2048",
    "page-snake",
    "page-minesweeper",
    "page-tetris",
    "page-noise",
];

const PAGE_TITLES: &[(&str, &str)] = &[
    ("page-login", "登录"),
    ("page-ggtt", "五笔查询"),
    ("page-dict", "辞典搜索"),
    ("page-qrcode", "二维码生成"),
    ("page-password", "密码生成"),
    ("page-uuid", "UUID"),
    ("page-base64", "Base64"),
    ("page-timestamp", "时间戳"),
    ("page-clipboard", "剪贴板历史"),
    ("page-notes", "笔记"),
    ("page-zici-chars", "生字表"),
    ("page-zici-words", "生词表"),
    ("page-search-history", "搜索历史"),
    ("page-short-notes", "短笔记"),
    ("page-songs", "音乐"),
    ("page-videos", "视频库"),
    ("page-g2048", "2048"),
    ("page-snake", "贪吃蛇"),
    ("page-minesweeper", "扫雷"),
    ("page-tetris", "俄罗斯方块"),
    ("page-noise", "白噪音"),
];

const SIDEBAR_IDS: &[&str] = &[
    "snav-ggtt",
    "snav-dict",
    "snav-qrcode",
    "snav-password",
    "snav-uuid",
    "snav-base64",
    "snav-timestamp",
    "snav-songs",
    "snav-zici-chars",
    "snav-zici-words",
    "snav-search-history",
    "snav-clipboard",
    "snav-notes",
    "snav-short-notes",
    "snav-videos",
    "snav-g2048",
    "snav-snake",
    "snav-minesweeper",
    "snav-tetris",
    "snav-noise",
];

fn show_page(page_id: &str) {
    for id in ALL_PAGES {
        if *id == page_id {
            unhide(id);
        } else {
            hide(id);
        }
    }
    // update title
    for (id, title) in PAGE_TITLES {
        if *id == page_id {
            el("page-title").set_text_content(Some(title));
            break;
        }
    }
    // sidebar active
    for sid in SIDEBAR_IDS {
        if let Some(snav) = doc().get_element_by_id(sid) {
            if let Ok(e) = snav.dyn_into::<HtmlElement>() {
                let target_page = sid.strip_prefix("snav-").unwrap_or("");
                let expected = format!("page-{}", target_page);
                if expected == page_id {
                    e.class_list().add_1("active").ok();
                } else if target_page != "logout" {
                    e.class_list().remove_1("active").ok();
                }
            }
        }
    }
}

fn navigate(hash: &str) {
    web_sys::window().unwrap().location().set_hash(hash).ok();
}

fn route() {
    let hash = web_sys::window()
        .unwrap()
        .location()
        .hash()
        .unwrap_or_default();
    let has_token = load_token().is_some();

    if !has_token {
        show_page("page-login");
        return;
    }

    match hash.as_str() {
        "#/ggtt" => show_page("page-ggtt"),
        "#/dict" => show_page("page-dict"),
        "#/qrcode" => show_page("page-qrcode"),
        "#/password" => show_page("page-password"),
        "#/uuid" => show_page("page-uuid"),
        "#/base64" => show_page("page-base64"),
        "#/timestamp" => show_page("page-timestamp"),
        "#/clipboard" | "#/clipboard-history" => show_page("page-clipboard"),
        "#/notes" => show_page("page-notes"),
        "#/zici-chars" | "#/zici/chars" => show_page("page-zici-chars"),
        "#/zici-words" | "#/zici/words" => show_page("page-zici-words"),
        "#/songs" => {
            show_page("page-songs");
        }
        "#/search-history" => show_page("page-search-history"),
        "#/short-notes" => show_page("page-short-notes"),
        "#/videos" => {
            do_video_load_list();
            show_page("page-videos");
        }
        "#/g2048" | "#/game2048" => {
            G2048.with(|g| *g.borrow_mut() = Game2048::new());
            set_text("g2048-status", "");
            render_g2048();
            show_page("page-g2048");
        }
        "#/snake" | "#/game-snake" => {
            SNAKE_G.with(|s| *s.borrow_mut() = Snake::new());
            set_text("snake-status", "");
            render_snake();
            snake_start();
            show_page("page-snake");
        }
        "#/minesweeper" | "#/game-minesweeper" => {
            MSWEEP.with(|m| *m.borrow_mut() = Minesweeper::new());
            render_ms();
            show_page("page-minesweeper");
        }
        "#/tetris" | "#/game-tetris" => {
            TETRIS_G.with(|t| *t.borrow_mut() = Tetris::new());
            set_text("tetris-status", "");
            render_tetris();
            tetris_start();
            show_page("page-tetris");
        }
        "#/noise" | "#/white-noise" => show_page("page-noise"),
        _ => {
            // default hub → ggtt
            navigate("#/ggtt");
            show_page("page-ggtt");
        }
    }
}

/// 同源 API 基地址：页面 origin（生产端口自动跟随；开发由 trunk proxy 转发）
fn default_base_url() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(origin) = window.location().origin() {
                return origin;
            }
        }
    }
    String::new()
}

fn get_client() -> ApiClient {
    CLIENT.with(|rc| {
        let b = rc.borrow();
        if let Some(client) = b.as_ref() {
            let mut c = ApiClient::new(&default_base_url());
            if let Some(t) = client.token() {
                c.set_token(t);
            }
            c
        } else {
            ApiClient::new(&default_base_url())
        }
    })
}

fn set_token_inner(token: &str) {
    CLIENT.with(|c| {
        let mut b = c.borrow_mut();
        let client = b.get_or_insert_with(|| ApiClient::new(&default_base_url()));
        if token.is_empty() {
            client.clear_token();
        } else {
            client.set_token(token);
        }
    });
}

#[wasm_bindgen]
pub fn set_token(token: &str) {
    set_token_inner(token);
}

// ==================== LOGIN ====================

async fn login_impl(user: &str, pass: &str) -> Result<String, String> {
    let client = get_client();
    match user_login(&client, user, pass).await {
        Ok(resp) => resp.data.map(|d| d.token).ok_or("无数据".into()),
        Err(e) => Err(format!("{}", e)),
    }
}

// ==================== GGTT ====================

async fn do_wubi_search() {
    disable_btn("wubi-btn", true);
    set_html("wubi-result", "");
    let req = SearchRequest {
        search: input("wubi-input"),
    };
    let client = get_client();
    match search_ggtt_code(&client, req).await {
        Ok(resp) => {
            if let Some(d) = resp.data {
                let mut svgs = String::new();
                for s in [&d.svg1, &d.svg2, &d.svg3, &d.svg4].into_iter().flatten() {
                    if !s.is_empty() && s.contains("<path") {
                        svgs.push_str(&format!("<div style='display:inline-block;width:75px;height:75px;margin:4px'>{}</div>", s));
                    }
                }
                let svg_section = if svgs.is_empty() {
                    String::new()
                } else {
                    format!("<div style='margin-top:12px'>{}</div>", svgs)
                };
                set_html(
                    "wubi-result",
                    &format!(
                        "<div style='color:#0078D4;font-size:18px;font-weight:600'>{} → {}</div>{}",
                        d.char, d.code_86, svg_section,
                    ),
                );
            } else {
                set_text("wubi-result", "无结果");
            }
        }
        Err(e) => set_text("wubi-result", &format!("失败: {}", e)),
    }
    disable_btn("wubi-btn", false);
}

// ==================== DICT ====================

async fn do_dict_search() {
    disable_btn("dict-btn", true);
    set_html("dict-result", "<div style='color:#888'>搜索中...</div>");
    let word = input("dict-input");
    if word.is_empty() {
        set_text("dict-result", "请输入搜索词语");
        disable_btn("dict-btn", false);
        return;
    }
    let dict_type = select_val("dict-select");
    let client = get_client();

    let result = match dict_type.as_str() {
        "xiandai" => match dict::search_xiandaihanyu(&client, &word).await {
            Ok(r) => format!(
                "【现代汉语词典】\n{}",
                serde_json::to_string_pretty(&r.data).unwrap_or_default()
            ),
            Err(e) => format!("失败: {}", e),
        },
        "collins" => match dict::search_collins(&client, &word).await {
            Ok(r) => format!(
                "【柯林斯词典】\n{}",
                serde_json::to_string_pretty(&r.data).unwrap_or_default()
            ),
            Err(e) => format!("失败: {}", e),
        },
        "ldoce" => match dict::search_ldoce(&client, &word).await {
            Ok(r) => format!(
                "【朗文词典】\n{}",
                serde_json::to_string_pretty(&r.data).unwrap_or_default()
            ),
            Err(e) => format!("失败: {}", e),
        },
        _ => {
            let mut out = String::new();
            match dict::search_xiandaihanyu(&client, &word).await {
                Ok(r) => out.push_str(&format!(
                    "【现代汉语词典】\n{}\n\n",
                    serde_json::to_string_pretty(&r.data).unwrap_or_default()
                )),
                Err(e) => out.push_str(&format!("【现代汉语词典】失败: {}\n\n", e)),
            }
            match dict::search_collins(&client, &word).await {
                Ok(r) => out.push_str(&format!(
                    "【柯林斯词典】\n{}\n\n",
                    serde_json::to_string_pretty(&r.data).unwrap_or_default()
                )),
                Err(e) => out.push_str(&format!("【柯林斯词典】失败: {}\n\n", e)),
            }
            match dict::search_ldoce(&client, &word).await {
                Ok(r) => out.push_str(&format!(
                    "【朗文词典】\n{}",
                    serde_json::to_string_pretty(&r.data).unwrap_or_default()
                )),
                Err(e) => out.push_str(&format!("【朗文词典】失败: {}", e)),
            }
            out
        }
    };

    set_text("dict-result", &result);
    disable_btn("dict-btn", false);
}

// ==================== QR CODE ====================

fn do_qr_generate() {
    let text = textarea("qr-input");
    if text.is_empty() {
        set_text("qr-result", "请输入文本");
        return;
    }
    match generate_qr_unicode(&text) {
        Ok(qr_unicode) => {
            set_html("qr-result", &format!("<pre style='font-size:10px;line-height:1;font-family:monospace;background:#1A1A1A;padding:8px;overflow:auto;text-align:center'>{}</pre>", qr_unicode));
        }
        Err(e) => set_text("qr-result", &format!("生成失败: {}", e)),
    }
    match qr_info(&text) {
        Ok((version, size)) => {
            el("qr-info")
                .set_text_content(Some(&format!("版本: {}, 尺寸: {}x{}", version, size, size)));
        }
        // 生成失败时同样给出提示，而不是展示残留的旧信息
        Err(e) => el("qr-info").set_text_content(Some(&format!("生成失败: {}", e))),
    }
}

// ==================== PASSWORD ====================

fn do_password_gen() {
    let len: usize = input("pw-length").parse().unwrap_or(16);
    set_text("pw-result", &generate_password(len));
}
fn do_password_strong() {
    let len: usize = input("pw-length").parse().unwrap_or(16);
    set_text("pw-result", &generate_strong_password(len));
}

// ==================== UUID ====================

fn do_uuid_v4() {
    set_text("uuid-result", &uuid::generate_uuid_v4());
}
fn do_uuid_v5() {
    set_text(
        "uuid-result",
        &uuid::generate_uuid_v5("6ba7b810-9dad-11d1-80b4-00c04fd430c8", "wasm-demo"),
    );
}
fn do_uuid_v6() {
    set_text("uuid-result", &uuid::generate_uuid_v6());
}
fn do_uuid_v7() {
    set_text("uuid-result", &uuid::generate_uuid_v7());
}
fn do_uuid_validate() {
    let u = input("uuid-validate-input");
    if uuid::validate_uuid(&u) {
        set_html(
            "uuid-validate-result",
            "<span style='color:#4CAF50'>✓ 有效的 UUID</span>",
        );
    } else {
        set_html(
            "uuid-validate-result",
            "<span style='color:#f44336'>✗ 无效的 UUID</span>",
        );
    }
}

// ==================== BASE64 ====================

fn do_b64_encode() {
    set_text("b64-result", &base64_encode(&textarea("b64-input")));
}
fn do_b64_decode() {
    match base64_decode(&textarea("b64-input")) {
        Some(d) => set_text("b64-result", &d),
        None => set_text("b64-result", "解码失败: 无效的 Base64"),
    }
}

// ==================== TIMESTAMP ====================

fn do_ts_now() {
    let local = timestamp::get_current_local_time();
    let utc = timestamp::get_current_utc_time();
    let ts = timestamp::get_current_timestamp();
    set_html(
        "ts-result",
        &format!(
            "<div>本地时间: {}</div><div>UTC 时间: {}</div><div>时间戳: {}</div>",
            local, utc, ts
        ),
    );
}
fn do_ts_utc() {
    set_text("ts-result", &timestamp::get_current_utc_time());
}
fn do_ts_timestamp() {
    set_text("ts-result", &timestamp::get_current_timestamp().to_string());
}
fn do_ts_to_local() {
    let ts: i64 = input("ts-input").parse().unwrap_or(0);
    set_text("ts-result", &timestamp::timestamp_to_local(ts));
}
fn do_ts_to_utc() {
    let ts: i64 = input("ts-input").parse().unwrap_or(0);
    set_text("ts-result", &timestamp::timestamp_to_utc(ts));
}
fn do_ts_parse() {
    match timestamp::local_to_timestamp(&input("ts-str-input")) {
        Some(ts) => set_text("ts-result", &ts.to_string()),
        None => set_text("ts-result", "解析失败，格式示例: 2025-01-01 12:00:00"),
    }
}

// ==================== CLIPBOARD ====================

async fn do_clipboard_refresh() {
    disable_btn("clip-btn", true);
    set_text("clip-result", "加载中...");
    let client = get_client();
    match get_clipboard_history(&client, Some(30), None, None).await {
        Ok(entries) => {
            if entries.is_empty() {
                set_text("clip-result", "暂无记录");
            } else {
                let mut html = String::from("<div style='font-size:13px'>");
                for (i, e) in entries.iter().enumerate() {
                    let content = e.text_content.as_deref().unwrap_or("");
                    let preview = if content.len() > 100 {
                        format!("{}...", &content[..content.floor_char_boundary(100)])
                    } else {
                        content.to_string()
                    };
                    let escaped = preview
                        .replace("&", "&amp;")
                        .replace("<", "&lt;")
                        .replace(">", "&gt;");
                    html.push_str(&format!(
                        "<div style='padding:8px;margin-bottom:4px;border-bottom:1px solid #333'>\
                         <div style='color:#888;font-size:11px'>#{} | {} | {}</div>\
                         <div style='margin-top:4px;word-break:break-all'>{}</div></div>",
                        i + 1,
                        e.entry_type,
                        e.created_at,
                        escaped,
                    ));
                }
                html.push_str("</div>");
                set_html("clip-result", &html);
            }
        }
        Err(e) => set_text("clip-result", &format!("获取失败: {}", e)),
    }
    disable_btn("clip-btn", false);
}

// ==================== NOTES ====================

async fn do_notes_refresh() {
    disable_btn("notes-refresh-btn", true);
    set_text("notes-result", "加载中...");
    let client = get_client();
    match list_notes(&client, Some(1), Some(30)).await {
        Ok(resp) => {
            if let Some(notes) = resp.data {
                if notes.is_empty() {
                    set_text("notes-result", "暂无笔记");
                } else {
                    let mut html = String::from("<div style='font-size:13px'>");
                    for (i, note) in notes.iter().enumerate() {
                        let content = note
                            .text
                            .as_deref()
                            .or(note.simple_text.as_deref())
                            .unwrap_or("");
                        let preview = if content.len() > 80 {
                            format!("{}...", &content[..content.floor_char_boundary(80)])
                        } else {
                            content.to_string()
                        };
                        let escaped = preview
                            .replace("&", "&amp;")
                            .replace("<", "&lt;")
                            .replace(">", "&gt;");
                        let created = note.created_at.format("%Y-%m-%d %H:%M").to_string();
                        html.push_str(&format!(
                            "<div style='padding:8px;margin-bottom:4px;border-bottom:1px solid #333'>\
                             <div style='color:#888;font-size:11px'>#{} | {}</div>\
                             <div style='margin-top:4px;word-break:break-all'>{}</div></div>",
                            i + 1, created, escaped,
                        ));
                    }
                    html.push_str("</div>");
                    set_html("notes-result", &html);
                }
            } else {
                set_text("notes-result", "暂无数据");
            }
        }
        Err(e) => set_text("notes-result", &format!("获取失败: {}", e)),
    }
    disable_btn("notes-refresh-btn", false);
}

// ==================== ZICI CHARS ====================

fn do_zici_show_chars(grade: usize, term: usize) {
    let client = CLIENT.with(|c| c.borrow().clone());
    set_text("zici-char-display", "加载中...");
    spawn_local(async move {
        let chars_list: Vec<String> = match &client {
            Some(client) => common::api::zici::zici_chars(client, grade as u32, term as u32)
                .await
                .ok()
                .and_then(|r| r.data)
                .unwrap_or_default(),
            None => Vec::new(),
        };
        if chars_list.is_empty() {
            set_text("zici-char-display", "无数据");
            return;
        }
        let chars_html: String = chars_list
            .iter()
            .map(|c| format!(
                "<span style='display:inline-block;padding:4px 6px;margin:2px;background:#2D2D2D;border-radius:4px;font-size:20px;cursor:pointer'>{}</span>",
                c
            ))
            .collect();
        set_html(
            "zici-char-display",
            &format!("<div style='line-height:2.5'>{}</div>", chars_html),
        );
        // 事件委托：点击生字查笔画（span 动态生成，不能逐个预绑定）
        let cb = Closure::wrap(Box::new(|e: web_sys::MouseEvent| {
            let target = e
                .target()
                .and_then(|t| t.dyn_into::<web_sys::HtmlElement>().ok());
            if let Some(el) = target {
                let char = el.text_content().unwrap_or_default();
                if !char.is_empty() && char.len() <= 4 {
                    do_hanzi_strokes(&char);
                }
            }
        }) as Box<dyn FnMut(_)>);
        if let Some(container) = doc().get_element_by_id("zici-char-display") {
            container
                .add_event_listener_with_callback("click", cb.as_ref().unchecked_ref())
                .ok();
        }
        cb.forget();
    });
}

// ==================== ZICI WORDS ====================

fn do_zici_word_search() {
    let q = input("zici-word-search");
    let client = CLIENT.with(|c| c.borrow().clone());
    set_text("zici-word-display", "加载中...");
    spawn_local(async move {
        let words: Vec<String> = match &client {
            Some(client) => common::api::zici::zici_words(client, &q, 1, 500)
                .await
                .ok()
                .and_then(|r| r.data)
                .and_then(|v| v["data"].as_array().cloned())
                .unwrap_or_default()
                .into_iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect(),
            None => Vec::new(),
        };
        let html: String = words.iter().enumerate().map(|(i, w)| {
            format!("<span style='display:inline-block;padding:4px 8px;margin:3px;background:#2D2D2D;border-radius:4px;font-size:14px'>{} <span style='color:#888'>({})</span></span>", w, i + 1)
        }).collect();
        if html.is_empty() {
            set_text("zici-word-display", "无匹配结果");
        } else {
            set_html("zici-word-display", &html);
        }
    });
}

// ==================== SEARCH HISTORY ====================

async fn do_search_history() {
    disable_btn("shistory-btn", true);
    set_text("shistory-result", "加载中...");
    let client = get_client();
    match dict::get_recent_history(&client, 30).await {
        Ok(resp) => {
            if let Some(history) = resp.data {
                if history.is_empty() {
                    set_text("shistory-result", "暂无搜索历史");
                } else {
                    let mut html = String::from("<div style='font-size:13px'>");
                    for h in history.iter() {
                        let escaped = h
                            .word
                            .replace("&", "&amp;")
                            .replace("<", "&lt;")
                            .replace(">", "&gt;");
                        let time_str = h.time.format("%Y-%m-%d %H:%M").to_string();
                        html.push_str(&format!(
                            "<div style='padding:6px 8px;margin-bottom:3px;border-bottom:1px solid #333'>\
                             <span style='color:#0078D4'>{}</span> \
                             <span style='color:#888;font-size:11px'>| {}</span></div>",
                            escaped, time_str,
                        ));
                    }
                    html.push_str("</div>");
                    set_html("shistory-result", &html);
                }
            } else {
                set_text("shistory-result", "暂无数据");
            }
        }
        Err(e) => set_text("shistory-result", &format!("获取失败: {}", e)),
    }
    disable_btn("shistory-btn", false);
}

// ==================== SHORT NOTES ====================

async fn do_short_notes_refresh() {
    disable_btn("sn-refresh-btn", true);
    set_text("sn-result", "加载中...");
    let client = get_client();
    match list_short_notes(&client, Some(1), Some(30)).await {
        Ok(resp) => {
            if let Some(notes) = resp.data {
                if notes.is_empty() {
                    set_text("sn-result", "暂无短笔记");
                } else {
                    let mut html = String::from("<div style='font-size:13px'>");
                    for (i, note) in notes.iter().enumerate() {
                        let content = note.content.as_deref().unwrap_or("");
                        let preview = if content.len() > 80 {
                            format!("{}...", &content[..content.floor_char_boundary(80)])
                        } else {
                            content.to_string()
                        };
                        let escaped = preview
                            .replace("&", "&amp;")
                            .replace("<", "&lt;")
                            .replace(">", "&gt;");
                        html.push_str(&format!(
                            "<div style='padding:8px;margin-bottom:4px;border-bottom:1px solid #333'>\
                             <div style='color:#888;font-size:11px'>#{} | {}</div>\
                             <div style='margin-top:4px;word-break:break-all'>{}</div></div>",
                            i + 1, note.created_at, escaped,
                        ));
                    }
                    html.push_str("</div>");
                    set_html("sn-result", &html);
                }
            } else {
                set_text("sn-result", "暂无数据");
            }
        }
        Err(e) => set_text("sn-result", &format!("获取失败: {}", e)),
    }
    disable_btn("sn-refresh-btn", false);
}

async fn do_short_note_create() {
    let content = textarea("sn-content");
    if content.trim().is_empty() {
        set_text("sn-result", "请输入内容");
        return;
    }
    disable_btn("sn-create-btn", true);
    let client = get_client();
    let req = CreateShortNoteRequest {
        content: Some(content),
        view_name: None,
    };
    match create_short_note(&client, req).await {
        Ok(resp) => {
            set_text("sn-result", &format!("创建成功: {}", resp.msg));
            if let Ok(e) = el("sn-content").dyn_into::<HtmlTextAreaElement>() {
                e.set_value("")
            }
            do_short_notes_refresh().await;
        }
        Err(e) => set_text("sn-result", &format!("创建失败: {}", e)),
    }
    disable_btn("sn-create-btn", false);
}

// ==================== SIDEBAR ====================

fn do_sidebar_toggle() {
    let sidebar = el("sidebar");
    if sidebar.class_list().contains("collapsed") {
        sidebar.class_list().remove_1("collapsed").ok();
    } else {
        sidebar.class_list().add_1("collapsed").ok();
    }
}

// ==================== SONGS ====================

fn songs_base_url() -> String {
    let url = CLIENT.with(|rc| rc.borrow().as_ref().map(|c| c.base_url().to_string()));
    url.unwrap_or_default()
}

fn songs_render_list() {
    SONG_LIST.with(|sl| {
        let list = sl.borrow();
        let count = list.len();
        el("songs-count").set_text_content(Some(&format!("共 {} 首歌曲", count)));

        if count == 0 {
            set_html("songs-list", "<div style='color:#888;padding:16px'>暂无歌曲，请先扫描</div>");
            return;
        }

        let mut html = String::new();
        for (i, song) in list.iter().enumerate() {
            let cover_url = song.cover_url.as_deref().unwrap_or("");
            let title = song.title.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;");
            let artist = song.artist.as_deref().unwrap_or("未知艺术家").replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;");
            let album = song.album.as_deref().unwrap_or("未知专辑").replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;");
            let cover_img = if !cover_url.is_empty() {
                format!("<img src='{}' class='w-10 h-10 rounded object-cover bg-[#333]' style='width:40px;height:40px;border-radius:4px;object-fit:cover;background:#333'>", cover_url)
            } else {
                "<div style='width:40px;height:40px;border-radius:4px;background:#333;display:flex;align-items:center;justify-content:center;font-size:18px'>🎵</div>".to_string()
            };
            html.push_str(&format!(
                "<div class='song-item' data-index='{}' style='display:flex;align-items:center;gap:10px;padding:8px;cursor:pointer;border-bottom:1px solid #333;transition:background .15s' \
                 onmouseover='this.style.background=\"#2D2D2D\"' onmouseout='this.style.background=\"\"'>\
                 {}\
                 <div style='flex:1;min-width:0'>\
                 <div style='font-size:13px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis'>{}</div>\
                 <div style='font-size:11px;color:#888;white-space:nowrap;overflow:hidden;text-overflow:ellipsis'>{} · {}</div>\
                 </div></div>",
                i, cover_img, title, artist, album,
            ));
        }
        set_html("songs-list", &html);
        // Bind click handlers on each song item
        let list_el = el("songs-list");
        for i in 0..list_el.children().length() {
            if let Some(child) = list_el.children().item(i) {
                let idx_str = child.get_attribute("data-index").unwrap_or_default();
                if let Ok(idx) = idx_str.parse::<usize>() {
                    let cb = Closure::wrap(Box::new(move || { do_song_play_by_index(idx); }) as Box<dyn FnMut()>);
                    child.add_event_listener_with_callback("click", cb.as_ref().dyn_ref().unwrap()).ok();
                    cb.forget();
                }
            }
        }
    });
}

async fn do_songs_list() {
    disable_btn("songs-refresh-btn", true);
    set_text("songs-list", "加载中...");
    let client = get_client();
    match get_all_songs(&client, Some(1), Some(200)).await {
        Ok(resp) => {
            if let Some(page) = resp.data {
                SONG_LIST.with(|sl| *sl.borrow_mut() = page.data);
                songs_render_list();
            } else {
                set_text("songs-list", "暂无数据");
            }
        }
        Err(e) => {
            set_text("songs-list", &format!("加载失败: {}", e));
        }
    }
    disable_btn("songs-refresh-btn", false);
}

async fn do_songs_scan() {
    disable_btn("songs-scan-btn", true);
    el("songs-scan-btn").set_text_content(Some("扫描中..."));
    let client = get_client();
    match scan_songs(&client).await {
        Ok(_) => {
            do_songs_list().await;
        }
        Err(e) => {
            set_text("songs-list", &format!("扫描失败: {}", e));
        }
    }
    el("songs-scan-btn").set_text_content(Some("扫描"));
    disable_btn("songs-scan-btn", false);
}

fn do_song_play_by_index(index: usize) {
    SONG_LIST.with(|sl| {
        let list = sl.borrow();
        if index >= list.len() {
            return;
        }
        let song = &list[index];
        SONG_INDEX.with(|si| *si.borrow_mut() = index);

        let player_bar = el("song-player-bar");
        player_bar.set_attribute("style", "display:block").ok();

        let base = songs_base_url();
        let file_url = format!("{}/api/songs/file/{}", base, song.id);
        let cover_url = song.cover_url.as_deref().unwrap_or("").to_string();

        el("song-current-title").set_text_content(Some(&song.title));
        let artist = song.artist.as_deref().unwrap_or("未知艺术家");
        let album = song.album.as_deref().unwrap_or("未知专辑");
        el("song-current-artist").set_text_content(Some(&format!("{} · {}", artist, album)));

        if !cover_url.is_empty() {
            if let Some(img) = doc().get_element_by_id("song-current-cover") {
                img.set_attribute("src", &cover_url).ok();
            }
        }

        if let Some(audio_el) = doc().get_element_by_id("song-player") {
            if let Ok(audio) = audio_el.dyn_into::<web_sys::HtmlAudioElement>() {
                audio.set_src(&file_url);
                let _ = audio.play();
                el("song-play-btn").set_text_content(Some("⏸"));
            }
        }
        ensure_spectrum();
        start_spectrum();
    });
}

fn do_song_play_pause() {
    if let Some(audio_el) = doc().get_element_by_id("song-player") {
        if let Ok(audio) = audio_el.dyn_into::<web_sys::HtmlAudioElement>() {
            if audio.paused() {
                let _ = audio.play();
                el("song-play-btn").set_text_content(Some("⏸"));
            } else {
                let _ = audio.pause();
                el("song-play-btn").set_text_content(Some("▶"));
            }
        }
    }
}

fn do_song_prev() {
    SONG_INDEX.with(|si| {
        let idx = si.borrow();
        if *idx > 0 {
            do_song_play_by_index(*idx - 1);
        }
    });
}

fn do_song_next() {
    SONG_INDEX.with(|si| {
        let idx = *si.borrow();
        SONG_LIST.with(|sl| {
            let len = sl.borrow().len();
            if idx + 1 < len {
                do_song_play_by_index(idx + 1);
            }
        });
    });
}

// ==================== SPECTRUM ====================

thread_local! {
    static SPECTRUM_CTX: RefCell<Option<web_sys::AudioContext>> = const { RefCell::new(None) };
    static SPECTRUM_ANALYSER: RefCell<Option<web_sys::AnalyserNode>> = const { RefCell::new(None) };
    static SPECTRUM_INT: RefCell<Option<i32>> = const { RefCell::new(None) };
    static SPECTRUM_CB: RefCell<Option<Closure<dyn FnMut()>>> = const { RefCell::new(None) };
    static SPECTRUM_BINS: RefCell<Vec<u16>> = const { RefCell::new(Vec::new()) };
}

fn ensure_spectrum() {
    if SPECTRUM_CTX.with(|c| c.borrow().is_some()) {
        return;
    }

    let ctx = match web_sys::AudioContext::new() {
        Ok(c) => c,
        Err(_) => {
            log("AudioContext 创建失败");
            return;
        }
    };
    let analyser = match ctx.create_analyser() {
        Ok(a) => a,
        Err(_) => {
            log("AnalyserNode 创建失败");
            return;
        }
    };
    analyser.set_fft_size(256);
    analyser.set_smoothing_time_constant(0.5);

    // Connect audio element → analyser → destination
    if let Some(audio_el) = doc().get_element_by_id("song-player") {
        if let Ok(audio) = audio_el.dyn_into::<web_sys::HtmlAudioElement>() {
            if let Ok(source) = ctx.create_media_element_source(&audio) {
                let _ = source.connect_with_audio_node(&analyser);
                let _ = analyser.connect_with_audio_node(&ctx.destination());
            }
        }
    }

    // Pre-calculate logarithmic frequency bin mapping
    let buffer_len = analyser.frequency_bin_count() as usize; // fftSize/2 = 128
    let sample_rate = ctx.sample_rate() as f64; // usually 48000
    let log_min = (20.0f64).ln();
    let log_max = (20000.0f64).ln();
    let mut bins = Vec::with_capacity(48);
    for i in 0..48 {
        let freq = (log_min + (log_max - log_min) * i as f64 / 47.0f64).exp();
        let bin = ((freq * 256.0 / sample_rate).floor() as u16).min(buffer_len as u16 - 1);
        bins.push(bin);
    }

    SPECTRUM_CTX.with(|c| *c.borrow_mut() = Some(ctx));
    SPECTRUM_ANALYSER.with(|a| *a.borrow_mut() = Some(analyser));
    SPECTRUM_BINS.with(|b| *b.borrow_mut() = bins);
}

fn start_spectrum() {
    // Ensure analyser exists
    if SPECTRUM_ANALYSER.with(|a| a.borrow().is_none()) {
        return;
    }
    stop_spectrum();

    let cb = Closure::wrap(Box::new(|| {
        spectrum_tick();
    }) as Box<dyn FnMut()>);

    if let Some(window) = web_sys::window() {
        if let Ok(id) = window.set_interval_with_callback_and_timeout_and_arguments_0(
            cb.as_ref().unchecked_ref(),
            50, // 20fps
        ) {
            SPECTRUM_INT.with(|si| *si.borrow_mut() = Some(id));
            SPECTRUM_CB.with(|sc| *sc.borrow_mut() = Some(cb));
        }
    }
}

fn stop_spectrum() {
    SPECTRUM_INT.with(|si| {
        if let Some(id) = si.borrow_mut().take() {
            if let Some(window) = web_sys::window() {
                window.clear_interval_with_handle(id);
            }
        }
    });
    SPECTRUM_CB.with(|sc| *sc.borrow_mut() = None);
    // Reset bars to zero
    for i in 0..48 {
        if let Some(bar) = doc().get_element_by_id(&format!("spb-{}", i)) {
            bar.set_attribute("style", "height:1%;background:hsl(260,80%,55%);opacity:0.4")
                .ok();
        }
    }
}

fn spectrum_tick() {
    let analyser = match SPECTRUM_ANALYSER.with(|a| a.borrow().as_ref().cloned()) {
        Some(a) => a,
        None => return,
    };
    let bins = SPECTRUM_BINS.with(|b| b.borrow().clone());
    if bins.is_empty() {
        return;
    }

    let buf_len = analyser.frequency_bin_count() as usize;
    let mut data = vec![0u8; buf_len];
    analyser.get_byte_frequency_data(&mut data);

    for (i, &bin) in bins.iter().take(48.min(bins.len())).enumerate() {
        let bin = bin as usize;
        let val = data.get(bin).copied().unwrap_or(0) as f64 / 255.0;
        let height = (val * 100.0f64).max(1.0);
        let hue = 260.0 + (i as f64 / 47.0) * 40.0;
        let lightness = 55.0 + val * 35.0;
        let opacity = 0.4 + val * 0.6;
        if let Some(bar) = doc().get_element_by_id(&format!("spb-{}", i)) {
            bar.set_attribute(
                "style",
                &format!(
                    "height:{}%;background:hsl({:.0},80%,{:.0}%);opacity:{:.2}",
                    height, hue, lightness, opacity,
                ),
            )
            .ok();
        }
    }
}

fn init_spectrum_bars() {
    // Create 48 bar divs in the spectrum container
    if let Some(container) = doc().get_element_by_id("spectrum-container") {
        for i in 0..48 {
            if let Ok(bar) = doc().create_element("div") {
                bar.set_id(&format!("spb-{}", i));
                bar.set_attribute("class", "flex-1 rounded-sm").ok();
                bar.set_attribute("style", "height:1%;background:hsl(260,80%,55%);opacity:0.4")
                    .ok();
                let _ = container.append_child(&bar);
            }
        }
    }
}

// ==================== START ====================

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();

    // hash routing
    let cb = Closure::wrap(Box::new(route) as Box<dyn FnMut()>);
    web_sys::window()
        .unwrap()
        .add_event_listener_with_callback("hashchange", cb.as_ref().unchecked_ref())
        .ok();
    cb.forget();

    // Restore token
    match load_token() {
        Some(token) => {
            log("token 已从 localStorage 恢复");
            set_token_inner(&token);
        }
        None => log("token 不存在"),
    }

    if load_token().is_none() {
        show_page("page-login");
    } else {
        let hash = web_sys::window()
            .unwrap()
            .location()
            .hash()
            .unwrap_or_default();
        if hash.is_empty() || hash == "#/login" || hash == "#/" {
            navigate("#/ggtt");
        }
        route();
    }

    // Sidebar toggle
    on_click("s-toggle", do_sidebar_toggle);

    // Sidebar navigation
    on_click("snav-ggtt", || navigate("#/ggtt"));
    on_click("snav-dict", || navigate("#/dict"));
    on_click("snav-qrcode", || navigate("#/qrcode"));
    on_click("snav-password", || navigate("#/password"));
    on_click("snav-uuid", || navigate("#/uuid"));
    on_click("snav-base64", || navigate("#/base64"));
    on_click("snav-timestamp", || navigate("#/timestamp"));
    on_click("snav-clipboard", || navigate("#/clipboard"));
    on_click("snav-notes", || navigate("#/notes"));
    on_click("snav-zici-chars", || navigate("#/zici-chars"));
    on_click("snav-zici-words", || navigate("#/zici-words"));
    on_click("snav-search-history", || navigate("#/search-history"));
    on_click("snav-short-notes", || navigate("#/short-notes"));
    on_click("snav-songs", || navigate("#/songs"));
    on_click("snav-logout", || {
        clear_token();
        navigate("#/login");
    });

    // Login
    on_click("login-btn", || {
        spawn_local(async {
            disable_btn("login-btn", true);
            set_text("login-msg", "登录中...");
            match login_impl(&input("login-user"), &input("login-pass")).await {
                Ok(t) => {
                    save_token(&t);
                    navigate("#/ggtt");
                }
                Err(e) => set_text("login-msg", &format!("失败: {}", e)),
            }
            disable_btn("login-btn", false);
        })
    });
    on_keydown("login-pass", "Enter", || {
        el("login-btn").click();
    });

    // GGTT
    on_keydown("wubi-input", "Enter", || {
        el("wubi-btn").click();
    });
    on_click("wubi-btn", || spawn_local(do_wubi_search()));

    // Dict
    on_keydown("dict-input", "Enter", || {
        el("dict-btn").click();
    });
    on_click("dict-btn", || spawn_local(do_dict_search()));

    // QR Code
    on_click("qr-btn", do_qr_generate);

    // Password
    on_click("pw-btn", do_password_gen);
    on_click("pw-strong-btn", do_password_strong);

    // UUID
    on_click("uuid-v4-btn", do_uuid_v4);
    on_click("uuid-v5-btn", do_uuid_v5);
    on_click("uuid-v6-btn", do_uuid_v6);
    on_click("uuid-v7-btn", do_uuid_v7);
    on_click("uuid-validate-btn", do_uuid_validate);

    // Base64
    on_click("b64-encode-btn", do_b64_encode);
    on_click("b64-decode-btn", do_b64_decode);

    // Timestamp
    on_click("ts-now-btn", do_ts_now);
    on_click("ts-utc-btn", do_ts_utc);
    on_click("ts-timestamp-btn", do_ts_timestamp);
    on_click("ts-to-local-btn", do_ts_to_local);
    on_click("ts-to-utc-btn", do_ts_to_utc);
    on_click("ts-parse-btn", do_ts_parse);

    // Clipboard
    on_click("clip-btn", || spawn_local(do_clipboard_refresh()));

    // Notes
    on_click("notes-refresh-btn", || spawn_local(do_notes_refresh()));

    // Zici chars — grade/term buttons
    for (g, t) in [
        (1, 1),
        (1, 2),
        (2, 1),
        (2, 2),
        (3, 1),
        (3, 2),
        (4, 1),
        (4, 2),
        (5, 1),
        (5, 2),
        (6, 1),
        (6, 2),
    ] {
        let id = format!("zc-{}-{}", g, t);
        on_click(&id, move || do_zici_show_chars(g, t));
    }

    // Zici words
    on_click("zici-word-search-btn", do_zici_word_search);
    on_keydown("zici-word-search", "Enter", || {
        el("zici-word-search-btn").click();
    });

    // ─── 视频 ───
    on_click("snav-videos", || navigate("#/videos"));
    on_click("video-refresh", do_video_load_list);

    // ─── 游戏 / 白噪音绑定 ───
    bind_game_keys();
    on_click("g2048-restart", || {
        G2048.with(|g| *g.borrow_mut() = Game2048::new());
        set_text("g2048-status", "");
        render_g2048();
    });
    on_click("snake-restart", || {
        SNAKE_G.with(|s| *s.borrow_mut() = Snake::new());
        set_text("snake-status", "");
        render_snake();
        snake_start();
    });
    on_click("ms-restart", || {
        MSWEEP.with(|m| *m.borrow_mut() = Minesweeper::new());
        render_ms();
    });
    on_click("tetris-restart", || {
        TETRIS_G.with(|t| *t.borrow_mut() = Tetris::new());
        set_text("tetris-status", "");
        render_tetris();
        tetris_start();
    });
    on_click("noise-white", || noise_play("white"));
    on_click("noise-pink", || noise_play("pink"));
    on_click("noise-brown", || noise_play("brown"));
    on_click("noise-rain", || noise_play("rain"));

    // Search history
    on_click("shistory-btn", || spawn_local(do_search_history()));

    // Short notes
    on_click("sn-refresh-btn", || spawn_local(do_short_notes_refresh()));
    on_click("sn-create-btn", || spawn_local(do_short_note_create()));

    // Init spectrum bars
    init_spectrum_bars();

    // Songs
    on_click("songs-refresh-btn", || spawn_local(do_songs_list()));
    on_click("songs-scan-btn", || spawn_local(do_songs_scan()));
    on_click("song-play-btn", do_song_play_pause);
    on_click("song-prev-btn", do_song_prev);
    on_click("song-next-btn", do_song_next);
}

// ==================== GAMES (纯 WASM Canvas 游戏，逻辑复用 common) ====================

thread_local! {
    static G2048: RefCell<Game2048> = RefCell::new(Game2048::new());
    static SNAKE_G: RefCell<Snake> = RefCell::new(Snake::new());
    static MSWEEP: RefCell<Minesweeper> = RefCell::new(Minesweeper::new());
    static TETRIS_G: RefCell<Tetris> = RefCell::new(Tetris::new());
    static GAME_SNAKE_TIMER: RefCell<Option<i32>> = const { RefCell::new(None) };
    static GAME_TETRIS_TIMER: RefCell<Option<i32>> = const { RefCell::new(None) };
}

fn canvas2d(id: &str) -> web_sys::CanvasRenderingContext2d {
    let canvas = doc()
        .get_element_by_id(id)
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap();
    canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap()
}

fn fill_cell(ctx: &web_sys::CanvasRenderingContext2d, x: f64, y: f64, w: f64, h: f64, color: &str) {
    ctx.set_fill_style(&JsValue::from_str(color));
    ctx.fill_rect(x, y, w, h);
}

// ─── 2048 ───
fn render_g2048() {
    let state = G2048.with(|g| g.borrow().clone());
    let ctx = canvas2d("g2048-canvas");
    ctx.clear_rect(0.0, 0.0, 320.0, 320.0);
    let colors = [
        "#CDC1B4", "#EEE4DA", "#EDE0C8", "#F2B179", "#F59563", "#F67C5F", "#F65E3B", "#EDCF72",
        "#EDCC61", "#EDC850", "#EDC53F", "#EDC22E",
    ];
    for r in 0..4 {
        for cc in 0..4 {
            let v = state.board[r][cc];
            let x = cc as f64 * 80.0 + 4.0;
            let y = r as f64 * 80.0 + 4.0;
            let ci = if v == 0 {
                0
            } else {
                (v.trailing_zeros() as usize).min(11)
            };
            fill_cell(&ctx, x, y, 72.0, 72.0, colors[ci]);
            if v > 0 {
                ctx.set_fill_style(&JsValue::from_str(if v <= 4 {
                    "#776E65"
                } else {
                    "#F9F6F2"
                }));
                ctx.set_font(&format!(
                    "bold {}px sans-serif",
                    if v >= 128 { 18 } else { 26 }
                ));
                ctx.set_text_align("center");
                ctx.set_text_baseline("middle");
                ctx.fill_text(&v.to_string(), x + 36.0, y + 38.0).ok();
            }
        }
    }
    set_text("g2048-score", &format!("得分: {}", state.score));
    if state.over {
        set_text("g2048-status", "游戏结束 · 按 R 重开");
    }
}

fn g2048_move(dir: &str) {
    let moved = G2048.with(|g| g.borrow_mut().move_dir(dir));
    if moved {
        render_g2048();
    }
}

fn g2048_key(key: &str) {
    match key {
        "ArrowUp" | "w" | "W" => g2048_move("up"),
        "ArrowDown" | "s" | "S" => g2048_move("down"),
        "ArrowLeft" | "a" | "A" => g2048_move("left"),
        "ArrowRight" | "d" | "D" => g2048_move("right"),
        "u" | "U" => {
            G2048.with(|g| {
                g.borrow_mut().undo();
            });
            render_g2048();
        }
        "r" | "R" => {
            G2048.with(|g| *g.borrow_mut() = Game2048::new());
            set_text("g2048-status", "");
            render_g2048();
        }
        _ => {}
    }
}

// ─── 贪吃蛇 ───
fn render_snake() {
    let ctx = canvas2d("snake-canvas");
    ctx.clear_rect(0.0, 0.0, 400.0, 400.0);
    SNAKE_G.with(|s| {
        let s = s.borrow();
        for i in 0..400 {
            let v = s.cells[i / 20][i % 20];
            if v == 0 {
                continue;
            }
            let x = (i % 20) as f64 * 20.0;
            let y = (i / 20) as f64 * 20.0;
            fill_cell(
                &ctx,
                x + 1.0,
                y + 1.0,
                18.0,
                18.0,
                if v == 1 { "#2E8B57" } else { "#E74C3C" },
            );
        }
        set_text("snake-score", &format!("得分: {}", s.score));
        if s.over {
            set_text("snake-status", "游戏结束 · 按 R 重开");
        }
    });
}

fn snake_tick_loop() {
    SNAKE_G.with(|s| s.borrow_mut().tick());
    render_snake();
    let over = SNAKE_G.with(|s| s.borrow().over);
    if over {
        if let Some(t) = GAME_SNAKE_TIMER.with(|t| *t.borrow()) {
            web_sys::window().unwrap().clear_interval_with_handle(t);
            GAME_SNAKE_TIMER.with(|t| *t.borrow_mut() = None);
        }
    }
}

fn snake_start() {
    if GAME_SNAKE_TIMER.with(|t| t.borrow().is_none()) {
        let cb = Closure::wrap(Box::new(snake_tick_loop) as Box<dyn FnMut()>);
        let f: &js_sys::Function = cb.as_ref().unchecked_ref();
        let handle = web_sys::window()
            .unwrap()
            .set_interval_with_callback_and_timeout_and_arguments_0(f, 200)
            .unwrap();
        GAME_SNAKE_TIMER.with(|t| *t.borrow_mut() = Some(handle));
        cb.forget();
    }
}

fn snake_key(key: &str) {
    match key {
        "ArrowUp" | "w" | "W" => SNAKE_G.with(|s| s.borrow_mut().set_dir("up")),
        "ArrowDown" | "s" | "S" => SNAKE_G.with(|s| s.borrow_mut().set_dir("down")),
        "ArrowLeft" | "a" | "A" => SNAKE_G.with(|s| s.borrow_mut().set_dir("left")),
        "ArrowRight" | "d" | "D" => SNAKE_G.with(|s| s.borrow_mut().set_dir("right")),
        "r" | "R" => {
            SNAKE_G.with(|s| *s.borrow_mut() = Snake::new());
            set_text("snake-status", "");
            render_snake();
        }
        _ => {}
    }
}

// ─── 扫雷 ───
fn render_ms() {
    MSWEEP.with(|m| {
        let state = m.borrow();
        if let Some(grid) = doc().get_element_by_id("ms-grid") {
            grid.set_inner_html("");
            for i in 0..81 {
                let cell = doc().create_element("div").unwrap();
                let revealed = state.revealed[i / 9][i % 9];
                let flagged = state.flagged[i / 9][i % 9];
                let v = state.cells[i / 9][i % 9];
                let style = if revealed {
                    if v == 9 {
                        "background:#C0392B;color:#fff;display:flex;align-items:center;justify-content:center;font-size:12px;aspect-ratio:1;"
                    } else if v > 0 {
                        "background:#E8E8E8;color:#2980B9;display:flex;align-items:center;justify-content:center;font-size:11px;font-weight:bold;aspect-ratio:1;"
                    } else {
                        "background:#E8E8E8;aspect-ratio:1;"
                    }
                } else {
                    "background:#5D6D7E;cursor:pointer;aspect-ratio:1;"
                };
                cell.set_attribute("style", style).ok();
                if flagged {
                    cell.set_text_content(Some("🚩"));
                } else if revealed && v == 9 {
                    cell.set_text_content(Some("💣"));
                } else if revealed && v > 0 {
                    cell.set_text_content(Some(&v.to_string()));
                }
                let idx = i;
                let click = Closure::wrap(Box::new(move || {
                    MSWEEP.with(|m| m.borrow_mut().click(idx % 9, idx / 9));
                    render_ms();
                }) as Box<dyn FnMut()>);
                let flag = Closure::wrap(Box::new(move || {
                    MSWEEP.with(|m| m.borrow_mut().toggle_flag(idx % 9, idx / 9));
                    render_ms();
                }) as Box<dyn FnMut()>);
                cell.add_event_listener_with_callback("click", click.as_ref().unchecked_ref()).ok();
                cell.add_event_listener_with_callback("contextmenu", flag.as_ref().unchecked_ref()).ok();
                click.forget();
                flag.forget();
                grid.append_child(&cell).ok();
            }
        }
        let s = if state.won {
            "🎉 胜利！".to_string()
        } else if state.over {
            "💥 踩雷了".to_string()
        } else {
            "点击翻开 · 右键插旗".to_string()
        };
        set_text("ms-status", &s);
    });
}

// ─── 俄罗斯方块 ───
fn render_tetris() {
    let ctx = canvas2d("tetris-canvas");
    ctx.clear_rect(0.0, 0.0, 200.0, 400.0);
    let colors = [
        "#1A1A1A", "#00BCD4", "#2196F3", "#FF9800", "#FFEB3B", "#4CAF50", "#9C27B0", "#F44336",
    ];
    TETRIS_G.with(|t| {
        let t = t.borrow();
        for i in 0..200 {
            let v = t.board[i / 10][i % 10];
            if v == 0 {
                continue;
            }
            let x = (i % 10) as f64 * 20.0;
            let y = (i / 10) as f64 * 20.0;
            fill_cell(
                &ctx,
                x + 1.0,
                y + 1.0,
                18.0,
                18.0,
                colors[(v as usize).min(7)],
            );
        }
        set_text("tetris-score", &format!("得分: {}", t.score));
        if t.over {
            set_text("tetris-status", "游戏结束 · 按 R 重开");
        }
    });
}

fn tetris_tick_loop() {
    TETRIS_G.with(|t| t.borrow_mut().tick());
    render_tetris();
    let over = TETRIS_G.with(|t| t.borrow().over);
    if over {
        if let Some(t) = GAME_TETRIS_TIMER.with(|t| *t.borrow()) {
            web_sys::window().unwrap().clear_interval_with_handle(t);
            GAME_TETRIS_TIMER.with(|t| *t.borrow_mut() = None);
        }
    }
}

fn tetris_start() {
    if GAME_TETRIS_TIMER.with(|t| t.borrow().is_none()) {
        let cb = Closure::wrap(Box::new(tetris_tick_loop) as Box<dyn FnMut()>);
        let f: &js_sys::Function = cb.as_ref().unchecked_ref();
        let handle = web_sys::window()
            .unwrap()
            .set_interval_with_callback_and_timeout_and_arguments_0(f, 500)
            .unwrap();
        GAME_TETRIS_TIMER.with(|t| *t.borrow_mut() = Some(handle));
        cb.forget();
    }
}

fn tetris_key(key: &str) {
    match key {
        "ArrowLeft" | "a" | "A" => TETRIS_G.with(|t| t.borrow_mut().move_piece("left")),
        "ArrowRight" | "d" | "D" => TETRIS_G.with(|t| t.borrow_mut().move_piece("right")),
        "ArrowDown" | "s" | "S" => {
            TETRIS_G.with(|t| t.borrow_mut().move_piece("down"));
            render_tetris();
        }
        "ArrowUp" | "w" | "W" => {
            TETRIS_G.with(|t| t.borrow_mut().move_piece("up"));
            render_tetris();
        }
        " " => {
            for _ in 0..20 {
                TETRIS_G.with(|t| t.borrow_mut().move_piece("down"));
            }
            render_tetris();
        }
        "r" | "R" => {
            TETRIS_G.with(|t| *t.borrow_mut() = Tetris::new());
            set_text("tetris-status", "");
            render_tetris();
        }
        _ => {}
    }
}

// ─── 全局键盘分发（游戏页激活时） ───
fn bind_game_keys() {
    let cb = Closure::wrap(Box::new(|e: web_sys::KeyboardEvent| {
        let hash = web_sys::window()
            .unwrap()
            .location()
            .hash()
            .unwrap_or_default();
        let key = e.key();
        match hash.as_str() {
            "#/g2048" | "#/game2048" => g2048_key(&key),
            "#/snake" | "#/game-snake" => snake_key(&key),
            "#/tetris" | "#/game-tetris" => tetris_key(&key),
            _ => {}
        }
    }) as Box<dyn FnMut(_)>);
    web_sys::window()
        .unwrap()
        .add_event_listener_with_callback("keydown", cb.as_ref().unchecked_ref())
        .ok();
    cb.forget();
}

// ==================== NOISE (Wasm 合成 PCM + WebAudio 循环) ====================

thread_local! {
    static NOISE_CTX: RefCell<Option<web_sys::AudioContext>> = const { RefCell::new(None) };
    static NOISE_SRC: RefCell<Option<web_sys::AudioBufferSourceNode>> = const { RefCell::new(None) };
    static NOISE_KIND: RefCell<String> = const { RefCell::new(String::new()) };
}

fn noise_play(kind: &str) {
    // 停掉当前
    if let Some(src) = NOISE_SRC.with(|s| s.borrow().clone()) {
        src.stop().ok();
        NOISE_SRC.with(|s| *s.borrow_mut() = None);
    }
    if NOISE_KIND.with(|k| k.borrow().as_str() == kind) {
        NOISE_KIND.with(|k| *k.borrow_mut() = String::new());
        set_text("noise-status", "已停止");
        return;
    }
    NOISE_KIND.with(|k| *k.borrow_mut() = kind.to_string());
    set_text("noise-status", "生成中...");

    let kind_owned = kind.to_string();
    wasm_bindgen_futures::spawn_local(async move {
        let pcm: Vec<f32> = match kind_owned.as_str() {
            "white" => synth_white_noise(10000, 48000, 1),
            "pink" => synth_pink_noise(10000, 48000, 1),
            "brown" => synth_brown_noise(10000, 48000, 1),
            _ => synth_rain_noise(10000, 48000, 1),
        };
        let ctx = match NOISE_CTX.with(|c| c.borrow().clone()) {
            Some(c) => c,
            None => {
                let c = web_sys::AudioContext::new().ok();
                if c.is_none() {
                    set_text("noise-status", "AudioContext 不可用");
                    return;
                }
                NOISE_CTX.with(|n| *n.borrow_mut() = c.clone());
                c.unwrap()
            }
        };
        let buffer = ctx.create_buffer(1, pcm.len() as u32, 48000.0).ok();
        if let Some(buf) = buffer {
            if let Ok(mut data) = buf.get_channel_data(0) {
                for (i, v) in pcm.iter().enumerate() {
                    data[i] = *v;
                }
            }
            let source = ctx.create_buffer_source().ok();
            if let Some(src) = source {
                src.set_buffer(Some(&buf));
                src.set_loop(true);
                src.connect_with_audio_node(&ctx.destination()).ok();
                src.start().ok();
                NOISE_SRC.with(|s| *s.borrow_mut() = Some(src));
                set_text("noise-status", "播放中（循环）");
            }
        }
    });
}

// ==================== VIDEOS (原生 HTML5 video，无任何 JS 库) ====================

fn do_video_load_list() {
    let client = CLIENT.with(|c| c.borrow().clone());
    set_text("video-count", "加载中...");
    spawn_local(async move {
        match client {
            Some(client) => {
                let res = common::api::videos::list_videos(&client, None, Some(1), Some(100)).await;
                match res {
                    Ok(ok) => {
                        let items = match ok.data {
                            Some(d) => d.data,
                            None => Vec::new(),
                        };
                        set_text("video-count", &format!("共 {} 个视频", items.len()));
                        if let Some(list) = doc().get_element_by_id("video-list") {
                            list.set_inner_html("");
                            for v in items {
                                let card = doc().create_element("div").unwrap();
                                card.set_attribute(
                                    "class",
                                    "bg-[#2D2D2D] rounded p-2 cursor-pointer hover:bg-[#3A3A3A]",
                                )
                                .ok();
                                let name = v.name.clone();
                                let path = v.path.clone();
                                let size_mb = v.size / 1024 / 1024;
                                let fmt_str = v.format.unwrap_or_default();
                                card.set_inner_html(&format!(
                                    "<div class='text-sm text-white overflow-hidden text-ellipsis whitespace-nowrap'>{}</div>\
                                     <div class='text-[11px] text-[#888]'>{}MB {}</div>",
                                    escape_html(&name), size_mb, fmt_str
                                ));
                                let click =
                                    Closure::wrap(Box::new(move || do_video_play(&path, &name))
                                        as Box<dyn FnMut()>);
                                card.add_event_listener_with_callback(
                                    "click",
                                    click.as_ref().unchecked_ref(),
                                )
                                .ok();
                                click.forget();
                                list.append_child(&card).ok();
                            }
                        }
                    }
                    Err(e) => set_text("video-count", &format!("加载失败: {}", e)),
                }
            }
            None => set_text("video-count", "未登录"),
        }
    });
}

fn do_video_play(path: &str, name: &str) {
    let base = default_base_url();
    let token = load_token().unwrap_or_default();
    let url = format!(
        "{}/api/files/serve?path={}&token={}",
        base,
        js_sys::encode_uri_component(path),
        token
    );
    if let Some(video) = doc().get_element_by_id("video-player") {
        if let Ok(v) = video.dyn_into::<web_sys::HtmlVideoElement>() {
            v.set_src(&url);
            let _ = v.play();
        }
    }
    log(&format!("播放: {}", name));
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

// ==================== HANZI STROKES (SVG 笔画展示) ====================

fn do_hanzi_strokes(char: &str) {
    let client = CLIENT.with(|c| c.borrow().clone());
    let char_owned = char.to_string();
    spawn_local(async move {
        let strokes: Vec<String> = match &client {
            Some(client) => {
                let resp = client
                    .get(&format!(
                        "/api/zici/hanzi/svg?char={}",
                        js_sys::encode_uri_component(&char_owned)
                    ))
                    .await
                    .ok();
                match resp {
                    Some(r) => {
                        let json = r.json::<serde_json::Value>().await.ok();
                        json.and_then(|v| v["data"]["strokes"].as_array().cloned())
                            .unwrap_or_default()
                            .into_iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .filter(|s| !s.is_empty())
                            .collect()
                    }
                    None => Vec::new(),
                }
            }
            None => Vec::new(),
        };
        if strokes.is_empty() {
            log(&format!("[hanzi] {} 无笔画数据", char_owned));
            return;
        }
        let paths: String = strokes
            .iter()
            .map(|s| format!("<path d='{}' fill='none' stroke='#333' stroke-width='14' stroke-linecap='round' stroke-linejoin='round'/>", s))
            .collect();
        let svg = format!(
            "<svg viewBox='0 0 1024 1024' xmlns='http://www.w3.org/2000/svg' style='width:280px;height:280px'>{}</svg>",
            paths
        );
        // 显示在生字页下方
        if let Some(container) = doc().get_element_by_id("zici-stroke-display") {
            container.set_inner_html(&svg);
        }
        log(&format!(
            "[hanzi] {} 笔画: {} 笔",
            char_owned,
            strokes.len()
        ));
    });
}
