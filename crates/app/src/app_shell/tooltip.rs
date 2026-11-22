//! A small label next to the pointer, for the buttons that are only a sign.

use gpui::{
    AnyView, App, Context, IntoElement, Render, SharedString, Window, div, prelude::*, rgb,
};

use crate::theme;

struct Tooltip(SharedString);

impl Render for Tooltip {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .px_2()
            .py_1()
            .rounded_sm()
            .bg(rgb(theme::SURFACE_ACTIVE))
            .border_1()
            .border_color(rgb(theme::BORDER))
            .text_xs()
            .text_color(rgb(theme::TEXT))
            .child(self.0.clone())
    }
}

/// For `.tooltip(...)`.
pub fn text(label: String) -> impl Fn(&mut Window, &mut App) -> AnyView + 'static {
    let label = SharedString::from(label);
    move |_, cx| cx.new(|_| Tooltip(label.clone())).into()
}
