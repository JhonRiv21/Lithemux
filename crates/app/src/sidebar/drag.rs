//! What travels with the pointer while a card or a category is dragged to
//! another place of its list.

use gpui::{Context, IntoElement, Render, SharedString, Window, div, prelude::*, rgb};

use crate::theme;

#[derive(Clone)]
pub(super) struct CardDrag {
    pub session_id: String,
    pub title: SharedString,
}

#[derive(Clone)]
pub(super) struct CategoryDrag {
    pub category_id: String,
    pub name: SharedString,
}

fn label(text: SharedString) -> impl IntoElement {
    div()
        .px_2()
        .py_1()
        .rounded_sm()
        .bg(rgb(theme::SURFACE_ACTIVE))
        .text_sm()
        .text_color(rgb(theme::TEXT))
        .child(text)
}

impl Render for CardDrag {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        label(self.title.clone())
    }
}

impl Render for CategoryDrag {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        label(self.name.clone())
    }
}
