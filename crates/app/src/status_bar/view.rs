use std::rc::Rc;

use gpui::{Context, Entity, IntoElement, Render, Window, div, prelude::*, px, rgb};

use super::memory_label::{memory_label, total};
use crate::app_shell::live::Live;
use crate::i18n::Texts;
use crate::theme;

const HEIGHT: f32 = 24.0;

pub struct StatusBar {
    live: Entity<Live>,
    texts: Rc<Texts>,
}

impl StatusBar {
    pub fn new(live: Entity<Live>, cx: &mut Context<Self>) -> Self {
        cx.observe(&live, |this, live, cx| {
            this.texts = Rc::clone(&live.read(cx).texts);
            cx.notify();
        })
        .detach();
        let texts = Rc::clone(&live.read(cx).texts);
        StatusBar { live, texts }
    }

    /// «Memory: 1.2 GB in 3 sessions», or nothing while none runs here.
    fn memory_text(&self, cx: &Context<Self>) -> Option<String> {
        let (bytes, sessions) = total(&self.live.read(cx).memory)?;
        let memory = memory_label(bytes);
        Some(
            self.texts
                .count("status_bar.memory", sessions, &[("memory", &memory)]),
        )
    }

    /// The notice of PLAN §3.2: with its button while some session can be
    /// resumed on the new version; with only busy ones left behind, just
    /// how many.
    fn render_update(&self, cx: &mut Context<Self>) -> Option<impl IntoElement> {
        let update = self.live.read(cx).claude_update().clone();
        if !update.is_available() {
            return None;
        }
        let colors = theme::colors();
        let notice = if update.idle.is_empty() {
            self.texts
                .count("status_bar.busy_on_old", update.busy.len(), &[])
        } else {
            self.texts.get("status_bar.new_claude")
        };
        let button = (!update.idle.is_empty()).then(|| {
            div()
                .id("resume-idle")
                .px_2()
                .rounded_sm()
                .bg(rgb(colors.accent))
                .text_color(rgb(colors.on_accent))
                .on_click(cx.listener(|this, _, _, cx| {
                    this.live
                        .update(cx, |live, cx| live.resume_idle_on_the_new_version(cx));
                }))
                .child(
                    self.texts
                        .count("status_bar.resume_idle", update.idle.len(), &[]),
                )
        });
        Some(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(div().text_color(rgb(colors.attention)).child(notice))
                .children(button),
        )
    }
}

impl Render for StatusBar {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = theme::colors();
        div()
            .flex()
            .flex_none()
            .items_center()
            .justify_between()
            .gap_4()
            .h(px(HEIGHT))
            .px_3()
            .bg(rgb(colors.sidebar))
            .border_t_1()
            .border_color(rgb(colors.border))
            .text_xs()
            .text_color(rgb(colors.text_muted))
            .child(div().children(self.memory_text(cx)))
            .children(self.render_update(cx))
    }
}
