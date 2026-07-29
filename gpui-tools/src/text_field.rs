use std::ops::Range;

use gpui::prelude::*;
use gpui::*;

actions!(
    text_field,
    [
        Backspace,
        Delete,
        Left,
        Right,
        SelectLeft,
        SelectRight,
        SelectAll,
        Home,
        End,
        Paste,
        Cut,
        Copy,
    ]
);

pub struct TextField {
    pub focus_handle: FocusHandle,
    pub content: SharedString,
    pub placeholder: String,
    selected_range: Range<usize>,
    selection_reversed: bool,
}

impl TextField {
    pub fn new(cx: &mut Context<Self>, placeholder: &str) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            content: SharedString::default(),
            placeholder: placeholder.to_string(),
            selected_range: 0..0,
            selection_reversed: false,
        }
    }

    pub fn text(&self) -> &str {
        &self.content
    }

    pub fn set_text(&mut self, text: &str) {
        self.content = SharedString::from(text.to_string());
        let len = self.content.len();
        self.selected_range = len..len;
    }

    fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    fn backspace(&mut self, _: &Backspace, _window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            let cursor = self.cursor_offset();
            if cursor == 0 {
                return;
            }
            self.selected_range = (cursor - 1)..cursor;
        }
        self.replace_text_in_range(None, "", _window, cx)
    }

    fn delete(&mut self, _: &Delete, _window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            let cursor = self.cursor_offset();
            if cursor >= self.content.len() {
                return;
            }
            self.selected_range = cursor..(cursor + 1);
        }
        self.replace_text_in_range(None, "", _window, cx)
    }

    fn left(&mut self, _: &Left, _: &mut Window, cx: &mut Context<Self>) {
        let cursor = self.cursor_offset();
        if cursor > 0 {
            self.move_to(cursor - 1, cx);
        }
    }

    fn right(&mut self, _: &Right, _: &mut Window, cx: &mut Context<Self>) {
        let cursor = self.cursor_offset();
        if cursor < self.content.len() {
            self.move_to(cursor + 1, cx);
        }
    }

    fn select_all(&mut self, _: &SelectAll, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0, cx);
        self.selected_range = 0..self.content.len();
        cx.notify();
    }

    fn home(&mut self, _: &Home, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0, cx);
    }

    fn end(&mut self, _: &End, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(self.content.len(), cx);
    }

    fn paste(&mut self, _: &Paste, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            self.replace_text_in_range(None, &text, _window, cx);
        }
    }

    fn copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.content[self.selected_range.clone()].to_string(),
            ));
        }
    }

    fn cut(&mut self, _: &Cut, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.content[self.selected_range.clone()].to_string(),
            ));
            self.replace_text_in_range(None, "", _window, cx)
        }
    }

    fn move_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        self.selected_range = offset..offset;
        cx.notify()
    }

    fn on_mouse_down(&mut self, _: &MouseDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.move_to(self.content.len(), cx);
    }
}

impl EntityInputHandler for TextField {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let text = self.content.to_string();
        let start = range_utf16.start.min(text.len());
        let end = range_utf16.end.min(text.len());
        actual_range.replace(start..end);
        Some(text[start..end].to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        if self.selected_range.is_empty() {
            None
        } else {
            Some(UTF16Selection {
                range: self.selected_range.clone(),
                reversed: self.selection_reversed,
            })
        }
    }

    fn marked_text_range(&self, _window: &mut Window, _cx: &mut Context<Self>) -> Option<Range<usize>> {
        None
    }

    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {}

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16.unwrap_or(self.selected_range.clone());
        self.content = (self.content[0..range.start].to_owned()
            + new_text
            + &self.content[range.end..])
            .into();
        let new_cursor = range.start + new_text.len();
        self.selected_range = new_cursor..new_cursor;
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _new_selected_range_utf16: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16.unwrap_or(self.selected_range.clone());
        self.content = (self.content[0..range.start].to_owned()
            + new_text
            + &self.content[range.end..])
            .into();
        self.selected_range = range.start + new_text.len()..range.start + new_text.len();
        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        _range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        Some(bounds)
    }

    fn character_index_for_point(
        &mut self,
        _point: Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        None
    }
}

impl Render for TextField {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let display = if self.content.is_empty() {
            SharedString::from(self.placeholder.clone())
        } else {
            self.content.clone()
        };

        let has_focus = self.focus_handle.is_focused(window);
        let show_cursor = has_focus && self.selected_range.is_empty();
        let cursor_pos = self.cursor_offset();

        let mut el = div()
            .key_context("TextField")
            .track_focus(&self.focus_handle(cx))
            .cursor(CursorStyle::IBeam)
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::delete))
            .on_action(cx.listener(Self::left))
            .on_action(cx.listener(Self::right))
            .on_action(cx.listener(Self::select_all))
            .on_action(cx.listener(Self::home))
            .on_action(cx.listener(Self::end))
            .on_action(cx.listener(Self::paste))
            .on_action(cx.listener(Self::cut))
            .on_action(cx.listener(Self::copy))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .relative()
            .bg(rgb(0x3a3a3a))
            .rounded(px(6.))
            .px(px(12.))
            .py(px(8.))
            .w_full()
            .text_size(px(16.))
            .line_height(px(24.))
            .text_color(if self.content.is_empty() {
                rgb(0x888888)
            } else {
                rgb(0xffffff)
            })
            .child(display);

        if show_cursor {
            let cursor_x = px(12.0 + cursor_pos as f32 * 8.0);
            el = el.child(
                div()
                    .absolute()
                    .left(cursor_x)
                    .top(px(8.))
                    .w(px(2.))
                    .h(px(24.))
                    .bg(rgb(0xffffff)),
            );
        }

        el
    }
}

impl Focusable for TextField {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
