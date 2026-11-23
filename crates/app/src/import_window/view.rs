//! The import window: one row per session cmux has, with its box, its
//! folder and the category it goes to, and the button that imports.

use std::rc::Rc;

use gpui::{
    AnyElement, ClickEvent, Context, Entity, IntoElement, Pixels, Point, Render, Window, div,
    prelude::*, rgb,
};
use lithemux_core::modules::category::Category;

use lithemux_core::modules::import::{Imported, Proposal};

use super::actions::{Failed, import, propose};
use super::state::{Choice, ImportForm};
use crate::app_shell::live::Live;
use crate::app_shell::title_bar::title_bar;
use crate::i18n::Texts;
use crate::theme;

/// The selector of one session, open.
pub(super) struct OpenChoices {
    pub(super) number: usize,
    pub(super) choices: Vec<Choice>,
    pub(super) at: Point<Pixels>,
}

pub struct ImportWindow {
    pub(super) live: Entity<Live>,
    pub(super) texts: Rc<Texts>,
    pub(super) form: Option<ImportForm>,
    /// What the proposal left out, in words.
    pub(super) summary: Option<String>,
    /// The last thing that happened: what was imported, or what failed.
    pub(super) said: Option<String>,
    pub(super) choosing: Option<OpenChoices>,
    /// Reading or importing, away from the interface thread.
    pub(super) busy: bool,
}

impl ImportWindow {
    pub fn new(live: Entity<Live>, _: &mut Window, cx: &mut Context<Self>) -> Self {
        let texts = Rc::clone(&live.read(cx).texts);
        let mut window = ImportWindow {
            live,
            texts,
            form: None,
            summary: None,
            said: None,
            choosing: None,
            busy: false,
        };
        window.propose(cx);
        window
    }

    /// Reads cmux's file away from the interface thread.
    fn propose(&mut self, cx: &mut Context<Self>) {
        let home = self.live.read(cx).home().clone();
        self.busy = true;
        cx.spawn(async move |this, cx| {
            let proposed = cx.background_spawn(async move { propose(&home) }).await;
            let _ = this.update(cx, |this, cx| this.show_proposal(proposed, cx));
        })
        .detach();
    }

    fn show_proposal(&mut self, proposed: Result<Proposal, Failed>, cx: &mut Context<Self>) {
        self.busy = false;
        match proposed {
            Ok(proposal) => {
                let count = |n: usize| n.to_string();
                self.summary = Some(self.texts.with(
                    "import.summary",
                    &[
                        ("already", &count(proposal.already_imported)),
                        ("without_id", &count(proposal.without_conversation_id)),
                        ("not_claude", &count(proposal.not_claude)),
                    ],
                ));
                let categories: Vec<Category> = self.live.read(cx).categories().to_vec();
                self.form = Some(ImportForm::new(&proposal, &categories));
            }
            Err(error) => {
                self.form = None;
                self.said = Some(self.failed(&error));
            }
        }
        cx.notify();
    }

    fn failed(&self, detail: &str) -> String {
        self.texts.with("import.failed", &[("detail", detail)])
    }

    /// Imports away from the interface thread, then reads cmux again.
    pub(super) fn import(&mut self, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        let Some(chosen) = self.form.as_ref().and_then(|form| form.chosen().ok()) else {
            return;
        };
        let home = self.live.read(cx).home().clone();
        self.busy = true;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let imported = cx
                .background_spawn(async move { import(&home, &chosen) })
                .await;
            let _ = this.update(cx, |this, cx| this.imported(imported, cx));
        })
        .detach();
    }

    fn imported(&mut self, imported: Result<Imported, Failed>, cx: &mut Context<Self>) {
        self.said = Some(match imported {
            Ok(imported) => self.texts.count("import.imported", imported.sessions, &[]),
            Err(error) => self.failed(&error),
        });
        self.live.update(cx, |live, cx| {
            live.reload(cx);
            cx.notify();
        });
        self.propose(cx);
    }

    pub(super) fn open_choices(
        &mut self,
        number: usize,
        event: &ClickEvent,
        cx: &mut Context<Self>,
    ) {
        let Some(form) = &self.form else {
            return;
        };
        self.choosing = Some(OpenChoices {
            number,
            choices: form.choices_for(number),
            at: event.position(),
        });
        cx.notify();
    }

    pub(super) fn choose(&mut self, number: usize, choice: &Choice, cx: &mut Context<Self>) {
        if let Some(form) = &mut self.form {
            form.set_destination(number, choice.destination.clone());
        }
        self.choosing = None;
        cx.notify();
    }
}

impl ImportWindow {
    fn render_list(&self, form: Option<&ImportForm>, cx: &mut Context<Self>) -> AnyElement {
        let mut list = div()
            .id("import-rows")
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .px_4()
            .overflow_y_scroll();
        match form {
            Some(form) if !form.rows.is_empty() => {
                for row in &form.rows {
                    list = list.child(self.render_row(form, row, cx));
                }
            }
            Some(_) => {
                list = list.child(
                    div()
                        .py_2()
                        .text_sm()
                        .child(self.texts.get("import.nothing")),
                );
            }
            None => {}
        }
        list.into_any_element()
    }
}

impl Render for ImportWindow {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = theme::colors();
        let form = self.form.take();
        let list = self.render_list(form.as_ref(), cx);
        let footer = self.render_footer(form.as_ref(), cx);
        self.form = form;
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(colors.window))
            .text_color(rgb(colors.text))
            .child(title_bar(self.texts.get("import.title")))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .p_4()
                    .text_sm()
                    .child(self.texts.get("import.intro"))
                    .children(self.summary.clone().map(|summary| {
                        div()
                            .text_xs()
                            .text_color(rgb(colors.text_muted))
                            .child(summary)
                    })),
            )
            .child(list)
            .child(footer)
            .children(self.render_choices(cx))
    }
}
