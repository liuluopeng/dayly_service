use common::api::{
    client::ApiClient,
    ggtt::{search_ggtt_code, SearchRequest},
    user::{user_login, LoginRequest},
};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Document, HtmlElement, HtmlInputElement, Storage};

static mut CLIENT: Option<ApiClient> = None;
const TOKEN_KEY: &str = "wasm_demo_token";

fn get_client() -> &'static mut ApiClient {
    unsafe {
        if CLIENT.is_none() {
            CLIENT = Some(ApiClient::new("http://localhost:23001"));
        }
        CLIENT.as_mut().unwrap()
    }
}

fn log(msg: &str) {
    web_sys::console::log_1(&msg.into());
}

fn storage() -> Option<Storage> {
    let w = web_sys::window()?;
    w.local_storage().ok()?
}

fn save_token(token: &str) {
    if let Some(s) = storage() { s.set_item(TOKEN_KEY, token).ok(); }
    set_token_inner(token);
}

fn load_token() -> Option<String> {
    let s = storage()?;
    let token = s.get_item(TOKEN_KEY).ok()??;
    if token.is_empty() { None } else { Some(token) }
}

fn clear_token() {
    if let Some(s) = storage() { s.remove_item(TOKEN_KEY).ok(); }
    set_token_inner("");
}

fn doc() -> Document { web_sys::window().unwrap().document().unwrap() }

fn el(id: &str) -> HtmlElement {
    doc().get_element_by_id(id).unwrap().dyn_into::<HtmlElement>().unwrap()
}

fn input(id: &str) -> String {
    doc().get_element_by_id(id).unwrap()
        .dyn_into::<HtmlInputElement>().unwrap()
        .value()
}

fn set_html(id: &str, html: &str) { doc().get_element_by_id(id).unwrap().set_inner_html(html); }
fn set_text(id: &str, text: &str) { doc().get_element_by_id(id).unwrap().set_text_content(Some(text)); }
fn hide(id: &str) { el(id).set_attribute("style", "display:none").ok(); }
fn unhide(id: &str) { el(id).set_attribute("style", "display:block").ok(); }

fn disable_btn(id: &str, disabled: bool) {
    if disabled { el(id).set_attribute("disabled", "true").ok(); }
    else { el(id).remove_attribute("disabled").ok(); }
}

fn on_keydown(id: &str, key: &str, mut f: impl FnMut() + 'static) {
    let cb = Closure::wrap(Box::new(move |e: web_sys::KeyboardEvent| {
        if &e.key() == key { f(); }
    }) as Box<dyn FnMut(_)>);
    doc().get_element_by_id(id).unwrap()
        .add_event_listener_with_callback("keydown", cb.as_ref().dyn_ref().unwrap()).ok();
    cb.forget();
}

fn on_click(id: &str, mut f: impl FnMut() + 'static) {
    let cb = Closure::wrap(Box::new(f) as Box<dyn FnMut()>);
    doc().get_element_by_id(id).unwrap()
        .add_event_listener_with_callback("click", cb.as_ref().dyn_ref().unwrap()).ok();
    cb.forget();
}

fn route() {
    if load_token().is_some() {
        hide("page-login");
        unhide("page-app");
    } else {
        unhide("page-login");
        hide("page-app");
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();

    // 恢复已保存的 token
    match load_token() {
        Some(token) => {
            log("token 已从 localStorage 恢复");
            set_token_inner(&token);
        }
        None => log("token 不存在"),
    }
    route();

    on_click("login-btn", || spawn_local(async {
        disable_btn("login-btn", true);
        set_text("login-msg", "登录中...");
        let token = login_impl(&input("login-user"), &input("login-pass")).await;
        disable_btn("login-btn", false);
        match token {
            Ok(t) => {
                save_token(&t);
                route();
            }
            Err(e) => set_text("login-msg", &format!("失败: {}", e)),
        }
    }));

    on_keydown("login-pass", "Enter", || { let _ = el("login-btn").click(); });
    on_keydown("wubi-input", "Enter", || { let _ = el("wubi-btn").click(); });

    on_click("wubi-btn", || spawn_local(async {
        disable_btn("wubi-btn", true);
        set_html("wubi-result", "");
        let client = get_client();
        let req = SearchRequest { search: input("wubi-input") };
        match search_ggtt_code(client, req).await {
            Ok(resp) => {
                if let Some(d) = resp.data {
                    let mut svgs = String::new();
                    for svg in [&d.svg1, &d.svg2, &d.svg3, &d.svg4] {
                        if let Some(s) = svg {
                            if !s.is_empty() && s.contains("<path") {
                                svgs.push_str(&format!("<div style='display:inline-block;width:75px;height:75px;margin:4px'>{}</div>", s));
                            }
                        }
                    }
                    let svg_section = if svgs.is_empty() { String::new() } else {
                        format!("<div style='margin-top:12px'>{}</div>", svgs)
                    };
                    set_html("wubi-result", &format!(
                        "<div style='color:#0078D4;font-size:18px;font-weight:600'>{} → {}</div>{}",
                        d.char, d.code_86, svg_section,
                    ));
                } else { set_text("wubi-result", "无结果"); }
            }
            Err(e) => {
                log(&format!("GGTT 查询失败: {}", e));
                set_text("wubi-result", &format!("失败: {}", e));
            },
        }
        disable_btn("wubi-btn", false);
    }));

    on_click("logout-btn", || {
        clear_token();
        route();
    });
}

fn set_token_inner(token: &str) {
    unsafe {
        if let Some(client) = CLIENT.as_mut() {
            if token.is_empty() { client.clear_token(); }
            else { client.set_token(token); }
        }
    }
}

#[wasm_bindgen]
pub fn set_token(token: &str) { set_token_inner(token); }

async fn login_impl(user: &str, pass: &str) -> Result<String, String> {
    let client = get_client();
    match user_login(client, user, pass).await {
        Ok(resp) => resp.data.map(|d| d.token).ok_or("无数据".into()),
        Err(e) => Err(format!("{}", e)),
    }
}
