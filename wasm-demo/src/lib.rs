use common::api::{
    client::ApiClient,
    ggtt::{search_ggtt_code, SearchRequest},
    user::{user_login, LoginRequest},
};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Document, HtmlElement, HtmlInputElement};

static mut CLIENT: Option<ApiClient> = None;

fn get_client() -> &'static mut ApiClient {
    unsafe {
        if CLIENT.is_none() {
            CLIENT = Some(ApiClient::new("http://localhost:23001"));
        }
        CLIENT.as_mut().unwrap()
    }
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

fn on_click(id: &str, mut f: impl FnMut() + 'static) {
    let cb = Closure::wrap(Box::new(f) as Box<dyn FnMut()>);
    doc().get_element_by_id(id).unwrap()
        .add_event_listener_with_callback("click", cb.as_ref().dyn_ref().unwrap()).ok();
    cb.forget();
}

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();

    on_click("login-btn", || spawn_local(async {
        disable_btn("login-btn", true);
        set_text("login-msg", "登录中...");
        let token = login_impl(&input("login-user"), &input("login-pass")).await;
        disable_btn("login-btn", false);
        match token {
            Ok(t) => {
                set_token_inner(&t);
                hide("page-login");
                unhide("page-app");
            }
            Err(e) => set_text("login-msg", &format!("失败: {}", e)),
        }
    }));

    on_click("wubi-btn", || spawn_local(async {
        disable_btn("wubi-btn", true);
        set_html("wubi-result", "");
        let code = input("wubi-input");
        let client = get_client();
        let req = SearchRequest { search: code };
        match search_ggtt_code(client, req).await {
            Ok(resp) => {
                if let Some(d) = resp.data {
                    let diagram = if d.has_diagram { "<div style='color:#888;font-size:13px;margin-top:4px'>含字根图</div>" } else { "" };
                    set_html("wubi-result", &format!(
                        "<div style='color:#0078D4;font-size:18px;font-weight:600'>{} → {}</div>{}",
                        &d.char,
                        &d.code_86,
                        diagram,
                    ));
                } else {
                    set_text("wubi-result", "无结果");
                }
            }
            Err(e) => set_text("wubi-result", &format!("失败: {}", e)),
        }
        disable_btn("wubi-btn", false);
    }));

    on_click("logout-btn", || {
        set_token_inner("");
        hide("page-app");
        unhide("page-login");
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
