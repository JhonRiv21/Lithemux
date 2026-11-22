//! What travels with the pointer while a tab or a divider is dragged.

use gpui::{Context, IntoElement, Render, SharedString, Window, div, prelude::*, px, rgb};

use crate::theme;

/// A tab on its way to another place or pane.
#[derive(Clone)]
pub(super) struct TabDrag {
    pub session_id: String,
    pub title: SharedString,
}

impl Render for TabDrag {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .px_2()
            .py_1()
            .rounded_sm()
            .bg(rgb(theme::colors().surface_active))
            .text_sm()
            .text_color(rgb(theme::colors().text))
            .child(self.title.clone())
    }
}

/// The divider of a split being moved. Nothing is painted for it: the
/// panes themselves follow the pointer.
#[derive(Clone, Copy)]
pub(super) struct DividerDrag {
    pub split: u32,
}

impl Render for DividerDrag {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size(px(0.0))
    }
}
