mod text_field;
use text_field::TextField;

use gpui::prelude::*;
use gpui::*;

use common::front_can_do::base64 as b64;
use common::front_can_do::password as pw;
use common::front_can_do::timestamp as ts;
use common::front_can_do::uuid as uid;

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Base64,
    Password,
    Timestamp,
    Uuid,
    Calculator,
}

impl Tab {
    fn all() -> &'static [(Tab, &'static str, &'static str)] {
        &[
            (Tab::Base64, "Base64", "\u{1f4ac}"),
            (Tab::Password, "Password", "\u{1f512}"),
            (Tab::Timestamp, "Timestamp", "\u{23f0}"),
            (Tab::Uuid, "UUID", "\u{1f3f7}"),
            (Tab::Calculator, "Calculator", "\u{1f522}"),
        ]
    }
}

pub struct GpuiTools {
    current_tab: Tab,

    b64_input: Entity<TextField>,
    b64_output: SharedString,
    b64_is_encoding: bool,

    pw_length: f32,
    pw_output: SharedString,
    pw_strong_output: SharedString,

    ts_timestamp: SharedString,
    ts_local: SharedString,
    ts_utc: SharedString,

    uuid_v4: SharedString,
    uuid_v7: SharedString,
    uuid_validate_input: Entity<TextField>,
    uuid_validate_output: SharedString,

    calc_a: Entity<TextField>,
    calc_b: Entity<TextField>,
    calc_result: (i32, i32, i32),
}

impl GpuiTools {
    fn new(cx: &mut Context<Self>) -> Self {
        let now = ts::get_current_timestamp();
        Self {
            current_tab: Tab::Base64,

            b64_input: cx.new(|cx| TextField::new(cx, "Enter text to encode/decode")),
            b64_output: SharedString::default(),
            b64_is_encoding: true,

            pw_length: 16.0,
            pw_output: SharedString::default(),
            pw_strong_output: SharedString::default(),

            ts_timestamp: now.to_string().into(),
            ts_local: ts::get_current_local_time().into(),
            ts_utc: ts::get_current_utc_time().into(),

            uuid_v4: uid::generate_uuid_v4().into(),
            uuid_v7: uid::generate_uuid_v7().into(),
            uuid_validate_input: cx.new(|cx| TextField::new(cx, "Enter UUID to validate")),
            uuid_validate_output: SharedString::default(),

            calc_a: cx.new(|cx| TextField::new(cx, "Number A")),
            calc_b: cx.new(|cx| TextField::new(cx, "Number B")),
            calc_result: (0, 0, 0),
        }
    }
}

impl Render for GpuiTools {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .size_full()
            .bg(rgb(0x1e1e1e))
            .child(self.render_sidebar(cx))
            .child(self.render_content(window, cx))
    }
}

impl GpuiTools {
    fn render_sidebar(&self, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .w(px(200.))
            .h_full()
            .bg(rgb(0x252526))
            .border_r_1()
            .border_color(rgb(0x3c3c3c))
            .child(
                div()
                    .px(px(16.))
                    .py(px(16.))
                    .text_lg()
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(0xcccccc))
                    .child("GPUI Tools"),
            )
            .child(div().h(px(1.)).bg(rgb(0x3c3c3c)))
            .children(Tab::all().iter().map(|&(tab, name, icon)| {
                let is_active = tab == self.current_tab;
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px(px(16.))
                    .py(px(10.))
                    .cursor_pointer()
                    .bg(if is_active {
                        rgb(0x37373d)
                    } else {
                        rgb(0x252526)
                    })
                    .text_color(if is_active {
                        rgb(0xffffff)
                    } else {
                        rgb(0x969696)
                    })
                    .hover(|s| {
                        if !is_active {
                            s.bg(rgb(0x2a2a2e))
                        } else {
                            s
                        }
                    })
                    .child(
                        div()
                            .text_color(if is_active {
                                rgb(0xffffff)
                            } else {
                                rgb(0x969696)
                            })
                            .child(icon),
                    )
                    .child(div().text_size(px(14.)).child(name))
            }))
    }

    fn render_content(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        match self.current_tab {
            Tab::Base64 => self.render_base64(window, cx).into_any_element(),
            Tab::Password => self.render_password(window, cx).into_any_element(),
            Tab::Timestamp => self.render_timestamp(window, cx).into_any_element(),
            Tab::Uuid => self.render_uuid(window, cx).into_any_element(),
            Tab::Calculator => self.render_calculator(window, cx).into_any_element(),
        }
    }

    fn render_base64(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let output = self.b64_output.clone();
        let is_encoding = self.b64_is_encoding;
        div()
            .flex()
            .flex_col()
            .gap_4()
            .p(px(24.))
            .size_full()
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(0xcccccc))
                    .child("Base64"),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        div()
                            .px(px(12.))
                            .py(px(6.))
                            .bg(if is_encoding {
                                rgb(0x0078d4)
                            } else {
                                rgb(0x3a3a3a)
                            })
                            .rounded(px(4.))
                            .cursor_pointer()
                            .text_color(rgb(0xcccccc))
                            .text_size(px(13.))
                            .child("Encode")
                            .on_mouse_up(
                                MouseButton::Left,
                                cx.listener(|this, _: &MouseUpEvent, _window, cx| {
                                    this.b64_is_encoding = true;
                                    cx.notify();
                                }),
                            ),
                    )
                    .child(
                        div()
                            .px(px(12.))
                            .py(px(6.))
                            .bg(if !is_encoding {
                                rgb(0x0078d4)
                            } else {
                                rgb(0x3a3a3a)
                            })
                            .rounded(px(4.))
                            .cursor_pointer()
                            .text_color(rgb(0xcccccc))
                            .text_size(px(13.))
                            .child("Decode")
                            .on_mouse_up(
                                MouseButton::Left,
                                cx.listener(|this, _: &MouseUpEvent, _window, cx| {
                                    this.b64_is_encoding = false;
                                    cx.notify();
                                }),
                            ),
                    ),
            )
            .child(self.b64_input.clone())
            .child(
                div()
                    .px(px(16.))
                    .py(px(8.))
                    .bg(rgb(0x0078d4))
                    .rounded(px(6.))
                    .cursor_pointer()
                    .text_color(rgb(0xffffff))
                    .text_size(px(14.))
                    .hover(|s| s.bg(rgb(0x1a8ad4)))
                    .child("Execute")
                    .on_mouse_up(
                        MouseButton::Left,
                        cx.listener(|this, _: &MouseUpEvent, _window, cx| {
                            let input = this.b64_input.read(cx).text().to_string();
                            let result = if this.b64_is_encoding {
                                b64::base64_encode(&input)
                            } else {
                                b64::base64_decode(&input).unwrap_or_default()
                            };
                            this.b64_output = result.into();
                            cx.notify();
                        }),
                    ),
            )
            .child(
                div()
                    .bg(rgb(0x1e1e1e))
                    .rounded(px(6.))
                    .p(px(12.))
                    .w_full()
                    .min_h(px(60.))
                    .text_color(rgb(0x4ec9b0))
                    .text_size(px(14.))
                    .child(output),
            )
    }

    fn render_password(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let pw_out = self.pw_output.clone();
        let pw_strong_out = self.pw_strong_output.clone();
        let length = self.pw_length;
        div()
            .flex()
            .flex_col()
            .gap_4()
            .p(px(24.))
            .size_full()
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(0xcccccc))
                    .child("Password Generator"),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .text_color(rgb(0x969696))
                            .child("Length:"),
                    )
                    .child(
                        div()
                            .text_color(rgb(0xffffff))
                            .child(format!("{}", length as u32)),
                    )
                    .child(
                        div()
                            .px(px(8.))
                            .py(px(4.))
                            .bg(rgb(0x3a3a3a))
                            .rounded(px(4.))
                            .cursor_pointer()
                            .child("-")
                            .on_mouse_up(
                                MouseButton::Left,
                                cx.listener(|this, _: &MouseUpEvent, _window, cx| {
                                    this.pw_length = (this.pw_length - 1.0).max(4.0);
                                    cx.notify();
                                }),
                            ),
                    )
                    .child(
                        div()
                            .px(px(8.))
                            .py(px(4.))
                            .bg(rgb(0x3a3a3a))
                            .rounded(px(4.))
                            .cursor_pointer()
                            .child("+")
                            .on_mouse_up(
                                MouseButton::Left,
                                cx.listener(|this, _: &MouseUpEvent, _window, cx| {
                                    this.pw_length = (this.pw_length + 1.0).min(64.0);
                                    cx.notify();
                                }),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        div()
                            .px(px(16.))
                            .py(px(8.))
                            .bg(rgb(0x0078d4))
                            .rounded(px(6.))
                            .cursor_pointer()
                            .text_color(rgb(0xffffff))
                            .child("Generate")
                            .on_mouse_up(
                                MouseButton::Left,
                                cx.listener(|this, _: &MouseUpEvent, _window, cx| {
                                    let len = this.pw_length as usize;
                                    this.pw_output = pw::generate_password(len).into();
                                    this.pw_strong_output =
                                        pw::generate_strong_password(len).into();
                                    cx.notify();
                                }),
                            ),
                    ),
            )
            .child(
                div()
                    .text_color(rgb(0x969696))
                    .text_size(px(13.))
                    .child("Random Password:"),
            )
            .child(
                div()
                    .bg(rgb(0x2d2d2d))
                    .rounded(px(4.))
                    .p(px(10.))
                    .text_color(rgb(0xce9178))
                    .font_family(SharedString::from("monospace"))
                    .child(pw_out),
            )
            .child(
                div()
                    .text_color(rgb(0x969696))
                    .text_size(px(13.))
                    .child("Strong Password:"),
            )
            .child(
                div()
                    .bg(rgb(0x2d2d2d))
                    .rounded(px(4.))
                    .p(px(10.))
                    .text_color(rgb(0xce9178))
                    .font_family(SharedString::from("monospace"))
                    .child(pw_strong_out),
            )
    }

    fn render_timestamp(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let ts_now = self.ts_timestamp.clone();
        let ts_local = self.ts_local.clone();
        let ts_utc = self.ts_utc.clone();
        div()
            .flex()
            .flex_col()
            .gap_4()
            .p(px(24.))
            .size_full()
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(0xcccccc))
                    .child("Timestamp"),
            )
            .child(
                div()
                    .bg(rgb(0x2d2d2d))
                    .rounded(px(6.))
                    .p(px(16.))
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(
                        div()
                            .text_color(rgb(0x969696))
                            .text_size(px(13.))
                            .child("Current Timestamp (seconds):"),
                    )
                    .child(
                        div()
                            .text_color(rgb(0x4ec9b0))
                            .text_size(px(18.))
                            .child(ts_now),
                    )
                    .child(div().h(px(1.)).bg(rgb(0x3c3c3c)))
                    .child(
                        div()
                            .text_color(rgb(0x969696))
                            .text_size(px(13.))
                            .child("Local Time:"),
                    )
                    .child(
                        div()
                            .text_color(rgb(0x569cd6))
                            .text_size(px(16.))
                            .child(ts_local),
                    )
                    .child(div().h(px(1.)).bg(rgb(0x3c3c3c)))
                    .child(
                        div()
                            .text_color(rgb(0x969696))
                            .text_size(px(13.))
                            .child("UTC Time:"),
                    )
                    .child(
                        div()
                            .text_color(rgb(0x569cd6))
                            .text_size(px(16.))
                            .child(ts_utc),
                    ),
            )
            .child(
                div()
                    .px(px(16.))
                    .py(px(8.))
                    .bg(rgb(0x0078d4))
                    .rounded(px(6.))
                    .cursor_pointer()
                    .text_color(rgb(0xffffff))
                    .child("Refresh")
                    .on_mouse_up(
                        MouseButton::Left,
                        cx.listener(|this, _: &MouseUpEvent, _window, cx| {
                            this.ts_timestamp = ts::get_current_timestamp().to_string().into();
                            this.ts_local = ts::get_current_local_time().into();
                            this.ts_utc = ts::get_current_utc_time().into();
                            cx.notify();
                        }),
                    ),
            )
    }

    fn render_uuid(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let v4 = self.uuid_v4.clone();
        let v7 = self.uuid_v7.clone();
        let validate_out = self.uuid_validate_output.clone();
        div()
            .flex()
            .flex_col()
            .gap_4()
            .p(px(24.))
            .size_full()
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(0xcccccc))
                    .child("UUID Generator"),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        div()
                            .px(px(16.))
                            .py(px(8.))
                            .bg(rgb(0x0078d4))
                            .rounded(px(6.))
                            .cursor_pointer()
                            .text_color(rgb(0xffffff))
                            .child("Generate v4")
                            .on_mouse_up(
                                MouseButton::Left,
                                cx.listener(|this, _: &MouseUpEvent, _window, cx| {
                                    this.uuid_v4 = uid::generate_uuid_v4().into();
                                    cx.notify();
                                }),
                            ),
                    )
                    .child(
                        div()
                            .px(px(16.))
                            .py(px(8.))
                            .bg(rgb(0x0078d4))
                            .rounded(px(6.))
                            .cursor_pointer()
                            .text_color(rgb(0xffffff))
                            .child("Generate v7")
                            .on_mouse_up(
                                MouseButton::Left,
                                cx.listener(|this, _: &MouseUpEvent, _window, cx| {
                                    this.uuid_v7 = uid::generate_uuid_v7().into();
                                    cx.notify();
                                }),
                            ),
                    ),
            )
            .child(
                div()
                    .text_color(rgb(0x969696))
                    .text_size(px(13.))
                    .child("UUID v4:"),
            )
            .child(
                div()
                    .bg(rgb(0x2d2d2d))
                    .rounded(px(4.))
                    .p(px(10.))
                    .text_color(rgb(0xce9178))
                    .font_family(SharedString::from("monospace"))
                    .child(v4),
            )
            .child(
                div()
                    .text_color(rgb(0x969696))
                    .text_size(px(13.))
                    .child("UUID v7:"),
            )
            .child(
                div()
                    .bg(rgb(0x2d2d2d))
                    .rounded(px(4.))
                    .p(px(10.))
                    .text_color(rgb(0xce9178))
                    .font_family(SharedString::from("monospace"))
                    .child(v7),
            )
            .child(div().h(px(1.)).bg(rgb(0x3c3c3c)))
            .child(
                div()
                    .text_color(rgb(0x969696))
                    .text_size(px(13.))
                    .child("Validate UUID:"),
            )
            .child(self.uuid_validate_input.clone())
            .child(
                div()
                    .px(px(16.))
                    .py(px(8.))
                    .bg(rgb(0x0078d4))
                    .rounded(px(6.))
                    .cursor_pointer()
                    .text_color(rgb(0xffffff))
                    .child("Validate")
                    .on_mouse_up(
                        MouseButton::Left,
                        cx.listener(|this, _: &MouseUpEvent, _window, cx| {
                            let input = this.uuid_validate_input.read(cx).text().to_string();
                            let valid = uid::validate_uuid(&input);
                            this.uuid_validate_output = if valid {
                                "\u{2705} Valid UUID".into()
                            } else {
                                "\u{274c} Invalid UUID".into()
                            };
                            cx.notify();
                        }),
                    ),
            )
            .child(
                div()
                    .text_color(rgb(0x4ec9b0))
                    .child(validate_out),
            )
    }

    fn render_calculator(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let (add, add22, mul) = self.calc_result;
        div()
            .flex()
            .flex_col()
            .gap_4()
            .p(px(24.))
            .size_full()
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(0xcccccc))
                    .child("Calculator"),
            )
            .child(
                div()
                    .text_color(rgb(0x969696))
                    .text_size(px(13.))
                    .child("Number A:"),
            )
            .child(self.calc_a.clone())
            .child(
                div()
                    .text_color(rgb(0x969696))
                    .text_size(px(13.))
                    .child("Number B:"),
            )
            .child(self.calc_b.clone())
            .child(
                div()
                    .px(px(16.))
                    .py(px(8.))
                    .bg(rgb(0x0078d4))
                    .rounded(px(6.))
                    .cursor_pointer()
                    .text_color(rgb(0xffffff))
                    .child("Calculate")
                    .on_mouse_up(
                        MouseButton::Left,
                        cx.listener(|this, _: &MouseUpEvent, _window, cx| {
                            let a_text = this.calc_a.read(cx).text().to_string();
                            let b_text = this.calc_b.read(cx).text().to_string();
                            let a: i32 = a_text.trim().parse().unwrap_or(0);
                            let b: i32 = b_text.trim().parse().unwrap_or(0);
                            this.calc_result = (a + b, a + b + 12, a * b);
                            cx.notify();
                        }),
                    ),
            )
            .child(
                div()
                    .bg(rgb(0x2d2d2d))
                    .rounded(px(6.))
                    .p(px(16.))
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .text_color(rgb(0x4ec9b0))
                            .child(format!("A + B = {}", add)),
                    )
                    .child(
                        div()
                            .text_color(rgb(0x4ec9b0))
                            .child(format!("A + B + 12 = {}", add22)),
                    )
                    .child(
                        div()
                            .text_color(rgb(0x4ec9b0))
                            .child(format!("A x B = {}", mul)),
                    ),
            )
    }
}

fn actions() -> Vec<KeyBinding> {
    vec![
        KeyBinding::new("backspace", text_field::Backspace, None),
        KeyBinding::new("delete", text_field::Delete, None),
        KeyBinding::new("left", text_field::Left, None),
        KeyBinding::new("right", text_field::Right, None),
        KeyBinding::new("shift-left", text_field::SelectLeft, None),
        KeyBinding::new("shift-right", text_field::SelectRight, None),
        KeyBinding::new("cmd-a", text_field::SelectAll, None),
        KeyBinding::new("cmd-v", text_field::Paste, None),
        KeyBinding::new("cmd-c", text_field::Copy, None),
        KeyBinding::new("cmd-x", text_field::Cut, None),
        KeyBinding::new("home", text_field::Home, None),
        KeyBinding::new("end", text_field::End, None),
    ]
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(800.), px(600.)), cx);
        cx.bind_keys(actions());
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(|_cx| GpuiTools::new(_cx)),
        )
        .unwrap();
    });
}
