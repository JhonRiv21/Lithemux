//! Paints the tabs of a pane.

use gpui::{AnyElement, Context, Div, ElementId, SharedString, Stateful, div, prelude::*, px, rgb};
use lithemux_core::modules::workspace::Pane;

use super::tab_drag::TabDrag;
use super::view::Workspace;
use crate::sidebar::status::{Status, status_of};
use crate::sidebar::status_text::status_color;
use crate::theme;

const TAB_BAR_HEIGHT: f32 = 30.0;
/// Room for the dot, a few letters with their ellipsis, and the close mark.
const MIN_TAB_WIDTH: f32 = 96.0;

/// What a tab needs from its session.
pub(super) struct TabInfo {
    pub session_id: String,
    pub title: SharedString,
    pub status: Status,
}

pub(super) fn id_of(kind: &str, pane: u32, name: &str) -> ElementId {
    ElementId::Name(format!("{kind}-{pane}-{name}").into())
}

/// The box of a tab: the one on show stands out.
fn tab_frame(pane_id: u32, session_id: &str, is_shown: bool) -> Stateful<Div> {
    div()
        .id(id_of("tab", pane_id, session_id))
        .flex()
        // A tab gives up width, down to a few letters and an ellipsis,
        // before the row has to scroll.
        .flex_initial()
        .items_center()
        .gap_1()
        .h_full()
        .px_2()
        .min_w(px(MIN_TAB_WIDTH))
        .max_w(px(220.0))
        .border_r_1()
        .border_color(rgb(theme::colors().border))
        .text_sm()
        .text_color(rgb(if is_shown {
            theme::colors().text
        } else {
            theme::colors().text_muted
        }))
        .when(is_shown, |tab| tab.bg(rgb(theme::colors().window)))
}

impl Workspace {
    pub(super) fn tab_info(&self, session_id: &str, cx: &Context<Self>) -> TabInfo {
        let live = self.live.read(cx);
        let session = live.session(session_id);
        TabInfo {
            session_id: session_id.to_string(),
            title: session
                .map(|session| session.display_title().to_string())
                .unwrap_or_default()
                .into(),
            status: session.map_or(Status::NotRunning, |session| {
                status_of(live.host.phase(session_id), session.state, session.profile)
            }),
        }
    }

    pub(super) fn render_tab_bar(&self, pane: &Pane, cx: &mut Context<Self>) -> impl IntoElement {
        let pane_id = pane.id;
        let mut tabs = Vec::new();
        for (index, session_id) in pane.tabs.iter().enumerate() {
            let info = self.tab_info(session_id, cx);
            tabs.push(self.render_tab(pane_id, index, index == pane.active, info, cx));
        }
        div()
            .flex()
            .flex_none()
            .items_center()
            .h(px(TAB_BAR_HEIGHT))
            .bg(rgb(theme::colors().sidebar))
            .border_b_1()
            .border_color(rgb(theme::colors().border))
            .child(
                div()
                    .id(("tabs", pane_id as usize))
                    .flex()
                    // In a narrow pane the tabs come first: the buttons
                    // give up their room before a tab loses its name.
                    .flex_auto()
                    .min_w(px(MIN_TAB_WIDTH))
                    .h_full()
                    .overflow_x_scroll()
                    .children(tabs),
            )
            .child(self.render_toolbar(pane_id, cx))
    }

    fn render_tab(
        &self,
        pane_id: u32,
        index: usize,
        is_shown: bool,
        info: TabInfo,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let (to_select, to_drop) = (info.session_id.clone(), info.session_id.clone());
        let drag = TabDrag {
            session_id: info.session_id.clone(),
            title: info.title.clone(),
        };
        tab_frame(pane_id, &info.session_id, is_shown)
            .hover(|style| style.bg(rgb(theme::colors().surface_hover)))
            .drag_over::<TabDrag>(|style, _, _, _| style.bg(rgb(theme::colors().surface_active)))
            .on_click(cx.listener(move |this, _, window, cx| {
                this.select_tab(&to_select, window, cx);
            }))
            .on_drag(drag, |drag, _, _, cx| cx.new(|_| drag.clone()))
            .on_drop(cx.listener(move |this, drag: &TabDrag, window, cx| {
                if drag.session_id != to_drop {
                    this.move_tab(&drag.session_id, pane_id, Some(index), window, cx);
                }
                cx.stop_propagation();
            }))
            .child(
                div()
                    .flex_none()
                    .size(px(7.0))
                    .rounded_full()
                    .bg(rgb(status_color(&info.status))),
            )
            .child(div().flex_1().min_w_0().truncate().child(info.title))
            .child(self.render_close_button(pane_id, &info.session_id, cx))
            .into_any_element()
    }

    fn render_close_button(
        &self,
        pane_id: u32,
        session_id: &str,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let to_close = session_id.to_string();
        div()
            .id(id_of("close", pane_id, session_id))
            .flex_none()
            .px_1()
            .rounded_sm()
            .text_color(rgb(theme::colors().text_faint))
            .hover(|style| {
                style
                    .bg(rgb(theme::colors().surface_active))
                    .text_color(rgb(theme::colors().text))
            })
            .on_click(cx.listener(move |this, _, window, cx| {
                this.close_tab(&to_close, window, cx);
                cx.stop_propagation();
            }))
            .tooltip(crate::app_shell::tooltip::text(
                self.texts.get("workspace.close_tab"),
            ))
            .child("×")
    }
}
