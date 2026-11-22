//! The strip at the top of a window, in the colors of the theme: the
//! system's own title bar does not follow the theme of the app, so the
//! windows hide it and paint this one under the traffic lights.

use gpui::{Div, MouseButton, MouseDownEvent, SharedString, div, prelude::*, px, rgb};

use crate::theme;

pub const HEIGHT: f32 = 30.0;
/// Room for the close, minimize and zoom buttons of macOS.
const TRAFFIC_LIGHTS: f32 = 78.0;

/// Drags the window; a double click zooms it, as a title bar does.
pub fn title_bar(title: impl Into<SharedString>) -> Div {
    div()
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .h(px(HEIGHT))
        .px(px(TRAFFIC_LIGHTS))
        .bg(rgb(theme::colors().sidebar))
        .border_b_1()
        .border_color(rgb(theme::colors().border))
        .text_sm()
        .text_color(rgb(theme::colors().text_muted))
        .on_mouse_down(MouseButton::Left, |event: &MouseDownEvent, window, _| {
            if event.click_count == 2 {
                window.zoom_window();
            } else {
                window.start_window_move();
            }
        })
        .child(title.into())
}
