//! Paints what comes and goes over the list: the footer with its button,
//! the forms and the menu.

use std::path::Path;

use gpui::{AnyElement, Context, Div, anchored, deferred, div, prelude::*, px, rgb};

use super::card_view::id_of;
use super::form::Form;
use super::menu::{MenuCommand, MenuItem};
use super::view::Sidebar;
use crate::theme;

impl Sidebar {
    pub(super) fn render_footer(&self, cx: &mut Context<Self>) -> AnyElement {
        let footer = div()
            .flex()
            .flex_col()
            .flex_none()
            .gap_1()
            .p_2()
            .border_t_1()
            .border_color(rgb(theme::colors().border));
        if let Some(form @ Form::NewCategory { .. }) = &self.form {
            return footer.child(self.render_form(form, cx)).into_any_element();
        }
        footer
            .child(
                self.button("new-category", self.texts.get("sidebar.new_category"))
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.start_new_category(window, cx);
                    })),
            )
            .into_any_element()
    }

    fn button(&self, name: &str, label: String) -> gpui::Stateful<gpui::Div> {
        div()
            .id(id_of("button", name))
            .px_2()
            .py_1()
            .rounded_sm()
            .bg(rgb(theme::colors().surface))
            .text_sm()
            .hover(|style| style.bg(rgb(theme::colors().surface_active)))
            .child(label)
    }

    pub(super) fn render_form(&self, form: &Form, cx: &mut Context<Self>) -> AnyElement {
        let mut column = div()
            .flex()
            .flex_col()
            .gap_1()
            .p_1()
            .rounded_md()
            .bg(rgb(theme::colors().surface))
            .child(form.field().clone());
        let confirm = match form {
            Form::NewCategory { .. } | Form::NewSession { .. } => "form.create",
            Form::RenameSession { .. } | Form::RenameCategory { .. } => "form.save",
        };
        if let Form::NewCategory { folder, .. } = form {
            column = self.with_folder_choice(column, folder.as_deref(), cx);
        }
        column
            .child(self.render_form_buttons(confirm, cx))
            .into_any_element()
    }

    /// The folder of a new category: which one, and the button to choose it.
    fn with_folder_choice(
        &self,
        column: Div,
        folder: Option<&Path>,
        cx: &mut Context<Self>,
    ) -> Div {
        let chosen = match folder {
            Some(folder) => self
                .texts
                .with("form.folder", &[("path", &folder.display().to_string())]),
            None => self.texts.get("form.no_folder"),
        };
        column
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(theme::colors().text_muted))
                    .child(chosen),
            )
            .child(
                self.button("choose-folder", self.texts.get("form.choose_folder"))
                    .on_click(cx.listener(|this, _, _, cx| this.choose_folder(cx))),
            )
    }

    fn render_form_buttons(&self, confirm: &str, cx: &mut Context<Self>) -> Div {
        div()
            .flex()
            .gap_1()
            .child(
                self.button("form-confirm", self.texts.get(confirm))
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.submit_form(window, cx);
                    })),
            )
            .child(
                self.button("form-cancel", self.texts.get("form.cancel"))
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.cancel_form(window, cx);
                    })),
            )
    }

    fn render_menu_entry(
        &self,
        index: usize,
        item: &MenuItem,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let command = item.command.clone();
        let is_delete = matches!(
            command,
            MenuCommand::DeleteSession(_) | MenuCommand::DeleteCategory(_)
        );
        div()
            .id(("menu-item", index))
            .px_3()
            .py_1()
            .text_sm()
            .text_color(rgb(if is_delete {
                theme::colors().danger
            } else {
                theme::colors().text
            }))
            .hover(|style| {
                style
                    .bg(rgb(theme::colors().accent))
                    .text_color(rgb(theme::colors().on_accent))
            })
            .on_click(cx.listener(move |this, _, window, cx| {
                this.run(command.clone(), window, cx);
            }))
            .child(self.texts.get(item.label))
            .into_any_element()
    }

    pub(super) fn render_menu(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let menu = self.menu.as_ref()?;
        let mut entries = Vec::new();
        for (index, item) in menu.items.iter().enumerate() {
            entries.push(self.render_menu_entry(index, item, cx));
        }
        let list = div()
            .flex()
            .flex_col()
            .py_1()
            .min_w(px(190.0))
            .rounded_md()
            .bg(rgb(theme::colors().surface))
            .border_1()
            .border_color(rgb(theme::colors().border))
            .shadow_md()
            .occlude()
            .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                this.menu = None;
                cx.notify();
            }))
            .children(entries);
        Some(
            deferred(
                anchored()
                    .position(menu.at)
                    .snap_to_window_with_margin(px(8.0))
                    .child(list),
            )
            .with_priority(1)
            .into_any_element(),
        )
    }
}
