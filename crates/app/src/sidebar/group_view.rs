//! Paints a category: its header and, under it, its cards.

use gpui::{AnyElement, Context, Div, MouseButton, MouseDownEvent, Stateful, div, prelude::*, rgb};
use lithemux_core::modules::session::Profile;

use super::card_view::id_of;
use super::drag::CategoryDrag;
use super::menu::{MenuCommand, category_menu};
use super::tree::{Group, Tree};
use super::view::Sidebar;
use crate::theme;

/// The row of a category's name.
fn category_frame(category_id: &str) -> Stateful<Div> {
    div()
        .id(id_of("category", category_id))
        .flex()
        .items_center()
        .gap_1()
        .px_1()
        .pt_2()
        .text_xs()
        .text_color(rgb(theme::colors().text_muted))
        .hover(|style| style.text_color(rgb(theme::colors().text)))
        .drag_over::<CategoryDrag>(|style, _, _, _| style.bg(rgb(theme::colors().surface_active)))
}

impl Sidebar {
    /// The whole list: the pinned sessions, then every category.
    pub(super) fn render_list(
        &self,
        tree: &Tree,
        in_focus: Option<&str>,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let mut list = div()
            .id("sidebar-list")
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .gap_1()
            .p_2()
            .overflow_y_scroll();
        if !tree.pinned.is_empty() {
            list = list.child(self.section_title(self.texts.get("sidebar.pinned")));
            for card in &tree.pinned {
                list = list.child(self.render_card(card, in_focus, false, cx));
            }
        }
        if tree.groups.is_empty() {
            list = list.child(self.hint(self.texts.get("sidebar.empty")));
        }
        for group in &tree.groups {
            list = list.child(self.render_group(group, in_focus, cx));
        }
        list
    }

    pub(super) fn section_title(&self, title: String) -> AnyElement {
        div()
            .px_1()
            .pt_1()
            .text_xs()
            .text_color(rgb(theme::colors().text_faint))
            .child(title)
            .into_any_element()
    }

    pub(super) fn hint(&self, text: String) -> AnyElement {
        div()
            .px_1()
            .py_2()
            .text_sm()
            .text_color(rgb(theme::colors().text_faint))
            .child(text)
            .into_any_element()
    }

    pub(super) fn render_group(
        &self,
        group: &Group,
        in_focus: Option<&str>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let mut column = div()
            .flex()
            .flex_col()
            .gap_1()
            .child(self.render_category(group, cx));
        if let Some(form) = &self.form
            && form.is_for_new_session_in(&group.category_id)
        {
            column = column.child(self.render_form(form, cx));
        }
        if group.collapsed {
            return column.into_any_element();
        }
        // The coordinator heads the category and the rest hang from it.
        if let Some(coordinator) = &group.header {
            column = column.child(self.render_card(coordinator, in_focus, false, cx));
        }
        let hangs = group.header.is_some();
        for card in &group.cards {
            column = column.child(self.render_card(card, in_focus, hangs, cx));
        }
        if group.header.is_none() && group.cards.is_empty() {
            column = column.child(self.hint(self.texts.get("sidebar.empty_category")));
        }
        column.into_any_element()
    }

    fn render_category(&self, group: &Group, cx: &mut Context<Self>) -> AnyElement {
        if let Some(form) = &self.form
            && form.renames_category(&group.category_id)
        {
            return self.render_form(form, cx);
        }
        let menu = category_menu(group);
        let (toggled, dropped_on) = (group.category_id.clone(), group.category_id.clone());
        let collapsed = group.collapsed;
        let drag = CategoryDrag {
            category_id: group.category_id.clone(),
            name: group.name.clone().into(),
        };
        category_frame(&group.category_id)
            .on_click(cx.listener(move |this, _, window, cx| {
                this.run(
                    MenuCommand::SetCollapsed(toggled.clone(), !collapsed),
                    window,
                    cx,
                );
            }))
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(move |this, event: &MouseDownEvent, _, cx| {
                    this.open_menu(menu.clone(), event.position, cx);
                }),
            )
            .on_drag(drag, |drag, _, _, cx| cx.new(|_| drag.clone()))
            .on_drop(cx.listener(move |this, drag: &CategoryDrag, _, cx| {
                let dragged = drag.category_id.clone();
                this.change(cx, |actions| {
                    actions.drop_category_on(&dragged, &dropped_on)
                });
            }))
            .child(if collapsed { "▸" } else { "▾" })
            .child(div().flex_1().truncate().child(group.name.to_uppercase()))
            .child(self.render_add_session(&group.category_id, cx))
            .into_any_element()
    }

    /// The `+` of a category: a new Claude session in it.
    fn render_add_session(&self, category_id: &str, cx: &mut Context<Self>) -> impl IntoElement {
        let added_to = category_id.to_string();
        div()
            .id(id_of("add-session", category_id))
            .px_1()
            .rounded_sm()
            .hover(|style| style.bg(rgb(theme::colors().surface_active)))
            .tooltip(crate::app_shell::tooltip::text(
                self.texts.get("menu.new_claude_session"),
            ))
            .on_click(cx.listener(move |this, _, window, cx| {
                cx.stop_propagation();
                let new = MenuCommand::NewSession(added_to.clone(), Profile::Claude);
                this.run(new, window, cx);
            }))
            .child("+")
    }
}
