//! Paints one pane: its tabs, its buttons and the terminal on show.

use gpui::{AnyElement, Context, Stateful, Window, div, prelude::*, px, rgb};
use lithemux_core::modules::session::Profile;
use lithemux_core::modules::workspace::{Axis, Pane};

use super::tab_drag::TabDrag;
use super::tab_view::{TabInfo, id_of};
use super::view::Workspace;
use crate::sidebar::status::Status;
use crate::sidebar::status_text::status_text;
use crate::theme;

impl Workspace {
    pub(super) fn render_pane(
        &self,
        pane_id: u32,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let Some(pane) = self.layout.pane(pane_id) else {
            return div().into_any_element();
        };
        let is_focused = self.layout.active_pane().id == pane_id;
        let has_many_panes = self.layout.panes_in_order().len() > 1;
        div()
            .flex()
            .flex_col()
            .size_full()
            .border_1()
            .border_color(rgb(if is_focused && has_many_panes {
                theme::ACCENT
            } else {
                theme::BORDER
            }))
            .capture_any_mouse_down(cx.listener(move |this, _, window, cx| {
                this.focus_pane(pane_id, window, cx);
            }))
            .on_drop(cx.listener(move |this, drag: &TabDrag, window, cx| {
                this.move_tab(&drag.session_id, pane_id, None, window, cx);
            }))
            .child(self.render_tab_bar(pane, cx))
            .child(self.render_content(pane, cx))
            .into_any_element()
    }

    pub(super) fn render_toolbar(&self, pane_id: u32, cx: &mut Context<Self>) -> impl IntoElement {
        let button = |name: &'static str, glyph: &'static str, label_key: &str| {
            div()
                .id(id_of("button", pane_id, name))
                .flex()
                .items_center()
                .justify_center()
                .w(px(26.0))
                .h(px(22.0))
                .rounded_sm()
                .text_sm()
                .text_color(rgb(theme::TEXT_MUTED))
                .tooltip(crate::app_shell::tooltip::text(self.texts.get(label_key)))
                .child(glyph)
        };
        let enabled = |button: Stateful<gpui::Div>| {
            button.hover(|style| {
                style
                    .bg(rgb(theme::SURFACE_ACTIVE))
                    .text_color(rgb(theme::TEXT))
            })
        };
        div()
            .flex()
            .flex_none()
            .items_center()
            .gap_1()
            .px_1()
            .child(
                enabled(button("terminal", "+", "workspace.new_terminal")).on_click(
                    cx.listener(move |this, _, window, cx| this.new_terminal(pane_id, window, cx)),
                ),
            )
            .child(
                enabled(button("right", "◫", "workspace.split_right")).on_click(cx.listener(
                    move |this, _, window, cx| this.split(pane_id, Axis::Horizontal, window, cx),
                )),
            )
            .child(
                enabled(button("down", "⊟", "workspace.split_down")).on_click(cx.listener(
                    move |this, _, window, cx| this.split(pane_id, Axis::Vertical, window, cx),
                )),
            )
            // Reserved for the browser pane (PLAN §3.7): there, and off.
            .child(button("browser", "◎", "workspace.browser").opacity(0.35))
    }

    fn render_content(&self, pane: &Pane, cx: &mut Context<Self>) -> AnyElement {
        let content = div().flex().flex_col().flex_1().min_h_0();
        let Some(session_id) = pane.active_session() else {
            return content
                .items_center()
                .justify_center()
                .text_sm()
                .text_color(rgb(theme::TEXT_FAINT))
                .child(self.texts.get("workspace.empty_pane"))
                .into_any_element();
        };
        let info = self.tab_info(session_id, cx);
        let (is_shell, is_running) = {
            let live = self.live.read(cx);
            let is_shell = live
                .session(session_id)
                .is_some_and(|session| session.profile == Profile::Shell);
            (is_shell, live.host.is_running(session_id))
        };
        // Whatever the database says of it, a session without a program
        // here gets the line that says so and the button to start it.
        let status_line = (!is_running).then(|| self.render_status_line(&info, is_shell, cx));
        let terminal = self.shown.get(session_id).map(|shown| shown.view.clone());
        content
            .children(status_line)
            .child(div().flex_1().min_h_0().p_1().children(terminal))
            .into_any_element()
    }

    /// Above a terminal that is not running: why, and how to start it.
    fn render_status_line(
        &self,
        info: &TabInfo,
        is_shell: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let can_start = !matches!(info.status, Status::Opening | Status::Closing);
        let label_key = if self.shown.contains_key(&info.session_id) && !is_shell {
            "workspace.reopen"
        } else {
            "workspace.open"
        };
        let session_id = info.session_id.clone();
        div()
            .flex()
            .flex_none()
            .items_center()
            .gap_2()
            .px_2()
            .py_1()
            .text_sm()
            .text_color(rgb(theme::TEXT_MUTED))
            .child(status_text(&info.status, &self.texts))
            .when(can_start, |line| {
                line.child(
                    div()
                        .id(id_of("start", 0, &info.session_id))
                        .px_2()
                        .rounded_sm()
                        .bg(rgb(theme::SURFACE_ACTIVE))
                        .text_color(rgb(theme::TEXT))
                        .hover(|style| style.bg(rgb(theme::ACCENT)))
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.reopen(&session_id, window, cx);
                        }))
                        .child(self.texts.get(label_key)),
                )
            })
    }
}
