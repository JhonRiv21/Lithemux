//! Paints a card: one session.

use gpui::{
    AnyElement, Context, Div, ElementId, MouseButton, MouseDownEvent, SharedString, Stateful, div,
    prelude::*, px, rgb,
};

use super::drag::CardDrag;
use super::menu::{MenuCommand, card_menu};
use super::port_chips::ports_shown;
use super::status_text::{card_status_text, status_color};
use super::tree::Card;
use super::view::Sidebar;
use crate::status_bar::memory_label::memory_label;
use crate::theme;

pub(super) fn id_of(kind: &str, name: &str) -> ElementId {
    ElementId::Name(format!("{kind}-{name}").into())
}

/// Waiting for the user stands out; then, the card of what is in focus.
fn border_of(card: &Card, is_in_focus: bool) -> u32 {
    if card.status.is_highlighted() {
        theme::colors().attention
    } else if is_in_focus {
        theme::colors().accent
    } else {
        theme::colors().sidebar
    }
}

impl Sidebar {
    pub(super) fn render_card(
        &self,
        card: &Card,
        in_focus: Option<&str>,
        hangs: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if let Some(form) = &self.form
            && form.renames_session(&card.session_id)
        {
            return self.render_form(form, cx);
        }
        let is_in_focus = in_focus == Some(card.session_id.as_str());
        let menu = card_menu(card);
        let (opened, dropped_on) = (card.session_id.clone(), card.session_id.clone());
        let drag = CardDrag {
            session_id: card.session_id.clone(),
            title: card.title.clone().into(),
        };
        let title_row = self.render_title_row(card);
        let status_row = self.render_status_row(card, cx);
        self.card_frame(card, is_in_focus, hangs)
            .hover(|style| style.bg(rgb(theme::colors().surface_hover)))
            .drag_over::<CardDrag>(|style, _, _, _| style.border_color(rgb(theme::colors().accent)))
            .on_click(cx.listener(move |this, _, window, cx| {
                this.run(MenuCommand::Open(opened.clone()), window, cx);
            }))
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(move |this, event: &MouseDownEvent, window, cx| {
                    this.open_menu(menu.clone(), event.position, window, cx);
                }),
            )
            .on_drag(drag, |drag, _, _, cx| cx.new(|_| drag.clone()))
            .on_drop(cx.listener(move |this, drag: &CardDrag, _, cx| {
                let dragged = drag.session_id.clone();
                this.change(cx, |actions| actions.drop_session_on(&dragged, &dropped_on));
            }))
            .child(title_row)
            .children(card.preview.clone().map(|preview| {
                div()
                    .truncate()
                    .text_xs()
                    .text_color(rgb(theme::colors().text_muted))
                    .child(preview)
            }))
            .child(status_row)
            .into_any_element()
    }

    /// The box of a card: where it sits and how it stands out.
    fn card_frame(&self, card: &Card, is_in_focus: bool, hangs: bool) -> Stateful<Div> {
        div()
            .id(id_of("card", &card.session_id))
            .flex()
            .flex_col()
            .gap(px(2.0))
            .px_2()
            .py_1()
            .when(hangs, |card| card.ml_3())
            .rounded_md()
            .border_1()
            .border_color(rgb(border_of(card, is_in_focus)))
            .bg(rgb(if is_in_focus {
                theme::colors().surface_active
            } else {
                theme::colors().surface
            }))
    }

    /// The dot of its status, the title, its tags and the unread count.
    fn render_title_row(&self, card: &Card) -> Div {
        div()
            .flex()
            .items_center()
            .gap_1()
            .child(
                div()
                    .flex_none()
                    .size(px(8.0))
                    .rounded_full()
                    .bg(rgb(status_color(&card.status))),
            )
            .child(
                div()
                    .flex_1()
                    .truncate()
                    .text_sm()
                    .child(SharedString::from(card.title.clone())),
            )
            .children(self.render_tags(card))
            .children((card.unread > 0).then(|| self.render_unread(card)))
    }

    /// «coordinator», «kept alive».
    fn render_tags(&self, card: &Card) -> Vec<Div> {
        let mut tags = Vec::new();
        if card.is_coordinator {
            tags.push(self.texts.get("sidebar.coordinator"));
        }
        if card.keep_alive {
            tags.push(self.texts.get("sidebar.keep_alive"));
        }
        let tag = |text: String| {
            div()
                .flex_none()
                .px_1()
                .rounded_sm()
                .bg(rgb(theme::colors().surface_active))
                .text_xs()
                .text_color(rgb(theme::colors().text_muted))
                .child(text)
        };
        tags.into_iter().map(tag).collect()
    }

    fn render_unread(&self, card: &Card) -> AnyElement {
        let label = self
            .texts
            .with("sidebar.unread", &[("count", &card.unread.to_string())]);
        div()
            .id(id_of("unread", &card.session_id))
            .flex_none()
            .px_1()
            .rounded_full()
            .bg(rgb(theme::colors().badge))
            .text_color(rgb(theme::colors().on_accent))
            .text_xs()
            .tooltip(crate::app_shell::tooltip::text(label))
            .child(card.unread.to_string())
            .into_any_element()
    }

    /// Its status in words, where it works and the ports it listens on.
    fn render_status_row(&self, card: &Card, cx: &mut Context<Self>) -> Div {
        let (ports, more) = ports_shown(&card.ports);
        let status_color = if card.status.is_highlighted() {
            theme::colors().attention
        } else {
            theme::colors().text_muted
        };
        div()
            .flex()
            .items_center()
            .gap_1()
            .text_xs()
            // The text gives way, with an ellipsis; the ports never do.
            .child(
                div()
                    .flex_shrink(1.0)
                    .min_w_0()
                    .truncate()
                    .text_color(rgb(status_color))
                    .child(card_status_text(&card.status, &self.texts)),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_color(rgb(theme::colors().text_faint))
                    .child(SharedString::from(card.cwd.clone())),
            )
            .children(card.memory.map(|bytes| {
                div()
                    .flex_none()
                    .text_color(rgb(theme::colors().text_faint))
                    .child(memory_label(bytes))
            }))
            .children(ports.iter().map(|port| self.render_port(card, *port, cx)))
            .children((more > 0).then(|| {
                div()
                    .flex_none()
                    .text_color(rgb(theme::colors().text_faint))
                    .child(format!("+{more}"))
            }))
    }

    /// `:3000`; a click shows that page in the browser.
    fn render_port(&self, card: &Card, port: u16, cx: &mut Context<Self>) -> AnyElement {
        div()
            .id(id_of("port", &format!("{}-{port}", card.session_id)))
            .flex_none()
            .px_1()
            .rounded_sm()
            .text_color(rgb(theme::colors().link))
            .hover(|style| style.bg(rgb(theme::colors().surface_active)))
            .on_click(cx.listener(move |this, _, _, cx| {
                cx.stop_propagation();
                this.open_port(port, cx);
            }))
            .child(format!(":{port}"))
            .into_any_element()
    }
}
