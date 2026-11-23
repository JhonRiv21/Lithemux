//! Paints the parts of the import window: a row per session, the selector
//! of a category and the footer with the button.

use gpui::{AnyElement, Context, IntoElement, anchored, deferred, div, prelude::*, px, rgb};

use super::state::{Choice, ImportForm, Row};
use super::view::ImportWindow;
use crate::sidebar::cwd_label::cwd_label;
use crate::theme;

impl ImportWindow {
    fn label_of(&self, choice: &Choice) -> String {
        if choice.is_new {
            self.texts
                .with("import.new_category", &[("name", &choice.name)])
        } else {
            choice.name.clone()
        }
    }

    pub(super) fn render_row(
        &self,
        form: &ImportForm,
        row: &Row,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let colors = theme::colors();
        div()
            .flex()
            .items_center()
            .gap_3()
            .py_1()
            .border_b_1()
            .border_color(rgb(colors.border))
            .child(self.render_mark(row, cx))
            .child(self.render_details(row))
            .child(self.render_category(form, row, cx))
            .into_any_element()
    }

    fn render_mark(&self, row: &Row, cx: &mut Context<Self>) -> impl IntoElement {
        let number = row.session.number;
        div()
            .id(("import-mark", number))
            .w(px(18.0))
            .flex_none()
            .cursor_pointer()
            .child(if row.marked { "☑" } else { "☐" })
            .on_click(cx.listener(move |this, _, _, cx| {
                if let Some(form) = &mut this.form {
                    form.toggle(number);
                }
                cx.notify();
            }))
    }

    /// Its name, and under it the folder and what to know about it.
    fn render_details(&self, row: &Row) -> impl IntoElement {
        let home = std::env::home_dir();
        let mut line = vec![cwd_label(&row.session.folder, home.as_deref())];
        if row.session.pinned {
            line.push(self.texts.get("import.pinned"));
        }
        if !row.session.has_conversation {
            line.push(self.texts.get("import.no_conversation"));
        }
        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w_0()
            .child(div().text_sm().truncate().child(row.session.name.clone()))
            .child(
                div()
                    .text_xs()
                    .truncate()
                    .text_color(rgb(theme::colors().text_muted))
                    .child(line.join(" · ")),
            )
    }

    fn render_category(
        &self,
        form: &ImportForm,
        row: &Row,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let colors = theme::colors();
        let number = row.session.number;
        let category = row
            .destination
            .as_ref()
            .and_then(|destination| form.choice_of(destination))
            .map_or_else(|| self.texts.get("import.choose"), |c| self.label_of(&c));
        div()
            .id(("import-category", number))
            .flex_none()
            .w(px(200.0))
            .px_2()
            .py_1()
            .rounded_sm()
            .truncate()
            .text_sm()
            .bg(rgb(colors.surface))
            .hover(|style| style.bg(rgb(colors.surface_active)))
            .when(row.destination.is_none() && row.marked, |button| {
                button.text_color(rgb(colors.attention))
            })
            .child(category)
            .on_click(cx.listener(move |this, event, _, cx| {
                this.open_choices(number, event, cx);
            }))
    }

    pub(super) fn render_choices(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let open = self.choosing.as_ref()?;
        let colors = theme::colors();
        let number = open.number;
        let entries = open.choices.iter().enumerate().map(|(index, choice)| {
            let chosen = choice.clone();
            div()
                .id(("import-choice", index))
                .px_3()
                .py_1()
                .text_sm()
                .hover(|style| {
                    style
                        .bg(rgb(colors.accent))
                        .text_color(rgb(colors.on_accent))
                })
                .child(self.label_of(choice))
                .on_click(cx.listener(move |this, _, _, cx| this.choose(number, &chosen, cx)))
        });
        let list = div()
            .id("import-choices")
            .flex()
            .flex_col()
            .py_1()
            .min_w(px(200.0))
            .max_h(px(320.0))
            .overflow_y_scroll()
            .rounded_md()
            .bg(rgb(colors.surface))
            .border_1()
            .border_color(rgb(colors.border))
            .shadow_md()
            .occlude()
            .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                this.choosing = None;
                cx.notify();
            }))
            .children(entries);
        Some(
            deferred(
                anchored()
                    .position(open.at)
                    .snap_to_window_with_margin(px(8.0))
                    .child(list),
            )
            .with_priority(1)
            .into_any_element(),
        )
    }

    pub(super) fn render_footer(
        &self,
        form: Option<&ImportForm>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let colors = theme::colors();
        let marked = form.map_or(0, ImportForm::marked);
        let ready = !self.busy && form.is_some_and(|form| form.chosen().is_ok()) && marked > 0;
        let hint = form
            .filter(|form| form.chosen().is_err())
            .map(|_| self.texts.get("import.needs_category"));
        div()
            .flex()
            .flex_none()
            .items_center()
            .justify_between()
            .gap_3()
            .p_4()
            .border_t_1()
            .border_color(rgb(colors.border))
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(colors.text_muted))
                    .children(hint.or_else(|| self.said.clone())),
            )
            .child(
                div()
                    .id("import-apply")
                    .flex_none()
                    .px_3()
                    .py_1()
                    .rounded_sm()
                    .text_sm()
                    .when(ready, |button| {
                        button
                            .bg(rgb(colors.accent))
                            .text_color(rgb(colors.on_accent))
                            .on_click(cx.listener(|this, _, _, cx| this.import(cx)))
                    })
                    .when(!ready, |button| {
                        button
                            .bg(rgb(colors.surface))
                            .text_color(rgb(colors.text_faint))
                    })
                    .child(self.texts.count("import.import", marked, &[])),
            )
            .into_any_element()
    }
}
