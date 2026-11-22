//! A one-line text field: gpui has none of its own.

use std::ops::Range;

use gpui::{
    App, Bounds, ClipboardItem, Context, ElementInputHandler, Entity, EntityInputHandler,
    EventEmitter, FocusHandle, Focusable, IntoElement, KeyDownEvent, Pixels, Point, Render,
    SharedString, UTF16Selection, Window, canvas, div, prelude::*, px, rgb,
};

use super::buffer::Buffer;
use crate::edit_actions::{Copy, Paste};
use crate::theme;

pub enum TextFieldEvent {
    Submitted,
    Cancelled,
}

pub struct TextField {
    buffer: Buffer,
    placeholder: SharedString,
    focus_handle: FocusHandle,
}

impl EventEmitter<TextFieldEvent> for TextField {}

impl TextField {
    pub fn new(text: &str, placeholder: String, cx: &mut Context<Self>) -> Self {
        TextField {
            buffer: Buffer::with_text(text),
            placeholder: placeholder.into(),
            focus_handle: cx.focus_handle(),
        }
    }

    pub fn set_text(&mut self, text: &str, cx: &mut Context<Self>) {
        self.buffer = Buffer::with_text(text);
        cx.notify();
    }

    pub fn text(&self) -> &str {
        self.buffer.text()
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        let keystroke = &event.keystroke;
        let command = keystroke.modifiers.platform;
        match keystroke.key.as_str() {
            "enter" => cx.emit(TextFieldEvent::Submitted),
            "escape" => cx.emit(TextFieldEvent::Cancelled),
            "backspace" => self.buffer.delete_before(),
            "delete" => self.buffer.delete_after(),
            "left" if command => self.buffer.start(),
            "right" if command => self.buffer.end(),
            "left" => self.buffer.left(),
            "right" => self.buffer.right(),
            "home" => self.buffer.start(),
            "end" => self.buffer.end(),
            // Text keys go to the input method.
            _ => return,
        }
        cx.stop_propagation();
        cx.notify();
    }
}

impl TextField {
    fn paste(&mut self, _: &Paste, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            self.buffer.replace(None, &text);
            cx.notify();
        }
    }

    fn copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        cx.write_to_clipboard(ClipboardItem::new_string(self.buffer.text().to_string()));
    }
}

impl Focusable for TextField {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EntityInputHandler for TextField {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        adjusted_range: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.buffer.range_from_utf16(&range_utf16);
        adjusted_range
            .replace(self.buffer.utf16_len(range.start)..self.buffer.utf16_len(range.end));
        Some(self.buffer.text()[range].to_string())
    }

    fn selected_text_range(
        &mut self,
        _: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        let cursor = self.buffer.cursor_utf16();
        Some(UTF16Selection {
            range: cursor..cursor,
            reversed: false,
        })
    }

    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        self.buffer.marked_utf16()
    }

    fn unmark_text(&mut self, _: &mut Window, _: &mut Context<Self>) {
        self.buffer.stop_composing();
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        text: &str,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16.map(|range| self.buffer.range_from_utf16(&range));
        self.buffer.replace(range, text);
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _: Option<Range<usize>>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16.map(|range| self.buffer.range_from_utf16(&range));
        self.buffer.compose(range, new_text);
        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        _: Range<usize>,
        element_bounds: Bounds<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        Some(element_bounds)
    }

    fn character_index_for_point(
        &mut self,
        _: Point<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<usize> {
        None
    }
}

/// Covers the field without painting: it is where the system delivers
/// typed text and compositions while the field has the focus.
fn input_layer(field: Entity<TextField>, focus_handle: FocusHandle) -> impl IntoElement {
    canvas(
        |_, _, _| {},
        move |bounds, (), window, cx| {
            let input = ElementInputHandler::new(bounds, field);
            window.handle_input(&focus_handle, input, cx);
        },
    )
    .absolute()
    .size_full()
}

impl Render for TextField {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focused = self.focus_handle.is_focused(window);
        let colors = theme::colors();
        let (before, after) = self.buffer.around_cursor();
        let is_empty = self.buffer.text().is_empty();
        div()
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::paste))
            .on_action(cx.listener(Self::copy))
            .on_key_down(cx.listener(Self::on_key_down))
            .relative()
            .flex()
            .items_center()
            .w_full()
            .h(px(24.0))
            .px_1()
            .overflow_hidden()
            .rounded_sm()
            .bg(rgb(colors.field))
            .border_1()
            .border_color(rgb(if focused {
                colors.accent
            } else {
                colors.border
            }))
            .text_sm()
            .text_color(rgb(colors.text))
            .child(SharedString::from(before.to_string()))
            .when(focused, |field| {
                field.child(div().w(px(1.0)).h(px(14.0)).bg(rgb(colors.text)))
            })
            .child(SharedString::from(after.to_string()))
            .when(is_empty, |field| {
                field.child(
                    div()
                        .text_color(rgb(colors.text_faint))
                        .child(self.placeholder.clone()),
                )
            })
            .child(input_layer(cx.entity(), self.focus_handle.clone()))
    }
}
