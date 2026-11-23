//! The palette as a gpui view: a field, the sessions it finds and which one
//! is selected. It says what the user chose; the window does the jumping.

use std::collections::HashMap;
use std::rc::Rc;

use gpui::{
    Context, Entity, EventEmitter, Focusable, IntoElement, KeyDownEvent, Render, ScrollHandle,
    Window, div, prelude::*, px, rgb,
};

use super::ranking::Entry;
use super::state::PaletteState;
use crate::i18n::Texts;
use crate::text_field::{TextField, TextFieldEvent};
use crate::theme;

const WIDTH: f32 = 560.0;

pub enum PaletteEvent {
    /// Show this session, waking it if it hibernates.
    Jump(String),
    Close,
}

pub struct Palette {
    state: PaletteState,
    field: Entity<TextField>,
    query: String,
    /// What each session is doing, in words.
    statuses: HashMap<String, String>,
    texts: Rc<Texts>,
    /// The list's scroll: the selected row stays in view.
    scroll: ScrollHandle,
}

impl EventEmitter<PaletteEvent> for Palette {}

impl Palette {
    pub fn new(
        entries: Vec<Entry>,
        statuses: HashMap<String, String>,
        texts: Rc<Texts>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let placeholder = texts.get("palette.placeholder");
        let field = cx.new(|cx| TextField::new("", placeholder, cx));
        cx.subscribe(&field, |this, _, event, cx| match event {
            TextFieldEvent::Submitted => this.jump(cx),
            TextFieldEvent::Cancelled => cx.emit(PaletteEvent::Close),
        })
        .detach();
        // The field repaints on every change: that is when to search again.
        cx.observe(&field, |this, field, cx| {
            let text = field.read(cx).text().to_string();
            if text != this.query {
                this.state.search(&text);
                this.scroll.scroll_to_item(0);
                this.query = text;
                cx.notify();
            }
        })
        .detach();
        window.focus(&field.focus_handle(cx), cx);
        Palette {
            state: PaletteState::new(entries),
            field,
            query: String::new(),
            statuses,
            texts,
            scroll: ScrollHandle::new(),
        }
    }

    fn jump(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = self.state.chosen() {
            cx.emit(PaletteEvent::Jump(id.to_string()));
        }
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        let keystroke = &event.keystroke;
        if keystroke.modifiers.modified() {
            return;
        }
        let step = match keystroke.key.as_str() {
            "up" => -1,
            "down" => 1,
            _ => return,
        };
        self.state.move_by(step);
        self.scroll.scroll_to_item(self.state.selected());
        cx.stop_propagation();
        cx.notify();
    }

    fn render_row(&self, index: usize, entry: &Entry, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = theme::colors();
        let is_selected = index == self.state.selected();
        let status = self
            .statuses
            .get(&entry.session_id)
            .cloned()
            .unwrap_or_default();
        div()
            .id(("palette-row", index))
            .flex()
            .justify_between()
            .gap_2()
            .px_3()
            .py_1()
            .text_sm()
            .when(is_selected, |row| {
                row.bg(rgb(colors.accent)).text_color(rgb(colors.on_accent))
            })
            .on_click(cx.listener(move |this, _, _, cx| {
                this.state.select(index);
                this.jump(cx);
            }))
            .child(div().truncate().child(entry.title.clone()))
            .child(
                div()
                    .flex_none()
                    .text_xs()
                    .when(!is_selected, |info| info.text_color(rgb(colors.text_muted)))
                    .child(format!("{} · {status}", entry.category)),
            )
    }
}

impl Render for Palette {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = theme::colors();
        let found: Vec<Entry> = self.state.found().to_vec();
        let rows: Vec<_> = found
            .iter()
            .enumerate()
            .map(|(index, entry)| self.render_row(index, entry, cx).into_any_element())
            .collect();
        div()
            .w(px(WIDTH))
            .flex()
            .flex_col()
            .gap_1()
            .p_2()
            .rounded_md()
            .bg(rgb(colors.surface))
            .border_1()
            .border_color(rgb(colors.border))
            .shadow_lg()
            .text_color(rgb(colors.text))
            .occlude()
            .on_key_down(cx.listener(Self::on_key_down))
            .on_mouse_down_out(cx.listener(|_, _, _, cx| cx.emit(PaletteEvent::Close)))
            .child(self.field.clone())
            .when(rows.is_empty(), |palette| {
                palette.child(
                    div()
                        .px_3()
                        .py_1()
                        .text_sm()
                        .text_color(rgb(colors.text_muted))
                        .child(self.texts.get("palette.nothing")),
                )
            })
            .child(
                div()
                    .id("palette-rows")
                    .flex()
                    .flex_col()
                    .max_h(px(420.0))
                    .overflow_y_scroll()
                    .track_scroll(&self.scroll)
                    .children(rows),
            )
    }
}
