use std::rc::Rc;

use gpui::{
    AnyElement, Context, ElementId, Entity, Focusable, IntoElement, Render, Window, div,
    prelude::*, px, rgb,
};
use lithemux_core::modules::launch::path_kind_on_disk;
use lithemux_core::modules::settings::{Preferences, SettingKey};

use super::form::{Choice, Input, Row, rows, value_of, warning_of};
use crate::app_shell::live::Live;
use crate::app_shell::title_bar::title_bar;
use crate::i18n::Texts;
use crate::text_field::{TextField, TextFieldEvent};
use crate::theme;

pub struct SettingsWindow {
    live: Entity<Live>,
    texts: Rc<Texts>,
    /// The typed settings, each with its field.
    fields: Vec<(SettingKey, Entity<TextField>)>,
    /// The settings the fields were last filled from.
    shown: Preferences,
}

impl SettingsWindow {
    pub fn new(live: Entity<Live>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        cx.observe_in(&live, window, |this, live, window, cx| {
            this.texts = Rc::clone(&live.read(cx).texts);
            this.follow_the_settings(window, cx);
            cx.notify();
        })
        .detach();
        let preferences = live.read(cx).preferences().clone();
        let typed = rows().into_iter().filter(|row| row.input == Input::Text);
        let fields = typed
            .map(|row| {
                let shown = value_of(&preferences, row.key);
                let field = cx.new(|cx| TextField::new(&shown, String::new(), cx));
                let key = row.key;
                cx.subscribe_in(
                    &field,
                    window,
                    move |this, field, event, _, cx| match event {
                        TextFieldEvent::Submitted => this.apply_typed(key, field, cx),
                        TextFieldEvent::Cancelled => this.show_value_in_use(key, field, cx),
                    },
                )
                .detach();
                (key, field)
            })
            .collect();
        let texts = Rc::clone(&live.read(cx).texts);
        SettingsWindow {
            live,
            texts,
            fields,
            shown: preferences,
        }
    }

    /// A setting changed from elsewhere (`lithemux config`, another
    /// window): its field shows the new value, unless the user is typing
    /// in it right now.
    fn follow_the_settings(&mut self, window: &Window, cx: &mut Context<Self>) {
        let preferences = self.live.read(cx).preferences().clone();
        if preferences == self.shown {
            return;
        }
        for (key, field) in &self.fields {
            let (before, now) = (value_of(&self.shown, *key), value_of(&preferences, *key));
            let is_being_typed = field.focus_handle(cx).is_focused(window);
            if before != now && !is_being_typed {
                field.update(cx, |field, cx| field.set_text(&now, cx));
            }
        }
        self.shown = preferences;
    }

    fn apply(&mut self, key: SettingKey, value: &str, cx: &mut Context<Self>) -> bool {
        let value = value.to_string();
        self.live
            .update(cx, |live, cx| live.set_setting(key, &value, cx))
    }

    /// Enter in a field: stored if it is accepted; either way the field
    /// ends up showing the value in use.
    fn apply_typed(&mut self, key: SettingKey, field: &Entity<TextField>, cx: &mut Context<Self>) {
        let typed = field.read(cx).text().to_string();
        if self.apply(key, &typed, cx) {
            self.show_value_in_use(key, field, cx);
        }
    }

    fn show_value_in_use(
        &mut self,
        key: SettingKey,
        field: &Entity<TextField>,
        cx: &mut Context<Self>,
    ) {
        let shown = value_of(self.live.read(cx).preferences(), key);
        field.update(cx, |field, cx| field.set_text(&shown, cx));
    }

    fn render_row(
        &self,
        row: &Row,
        current: &str,
        warning: Option<String>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let colors = theme::colors();
        let input = match &row.input {
            Input::Choices(choices) => self.render_choices(row.key, choices, current, cx),
            Input::Text => {
                let field = self.fields.iter().find(|(key, _)| *key == row.key);
                div()
                    .w(px(260.0))
                    .children(field.map(|(_, field)| field.clone()))
                    .into_any_element()
            }
        };
        div()
            .flex()
            .items_start()
            .gap_4()
            .py_2()
            .border_b_1()
            .border_color(rgb(colors.border))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .gap_1()
                    .child(div().text_sm().child(self.texts.get(row.label)))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(colors.text_muted))
                            .child(self.texts.get(row.hint)),
                    )
                    .children(warning.map(|warning| {
                        div()
                            .text_xs()
                            .text_color(rgb(colors.attention))
                            .child(warning)
                    })),
            )
            .child(input)
            .into_any_element()
    }

    /// The values to choose from, the one in use standing out.
    fn render_choices(
        &self,
        key: SettingKey,
        choices: &[Choice],
        current: &str,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let colors = theme::colors();
        let buttons = choices.iter().map(|choice| {
            let is_current = choice.value == current;
            let value = choice.value;
            div()
                .id(ElementId::Name(
                    format!("choice-{}-{value}", key.as_str()).into(),
                ))
                .px_3()
                .py_1()
                .rounded_sm()
                .text_sm()
                .bg(rgb(if is_current {
                    colors.accent
                } else {
                    colors.surface
                }))
                .text_color(rgb(if is_current {
                    colors.on_accent
                } else {
                    colors.text
                }))
                .when(!is_current, |button| {
                    button.hover(|style| style.bg(rgb(theme::colors().surface_active)))
                })
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.apply(key, value, cx);
                }))
                .child(self.texts.get(choice.label))
        });
        div()
            .flex()
            .flex_none()
            .gap_1()
            .children(buttons.collect::<Vec<_>>())
            .into_any_element()
    }
}

impl Render for SettingsWindow {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = theme::colors();
        let (preferences, notice) = {
            let live = self.live.read(cx);
            (live.preferences().clone(), live.notice.clone())
        };
        let mut list = div()
            .id("settings-rows")
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .px_4()
            .overflow_y_scroll();
        for row in rows() {
            let current = value_of(&preferences, row.key);
            let warning = warning_of(&preferences, row.key, path_kind_on_disk)
                .map(|warning| self.texts.say(&warning));
            list = list.child(self.render_row(&row, &current, warning, cx));
        }
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(colors.window))
            .text_color(rgb(colors.text))
            .child(title_bar(self.texts.get("settings.title")))
            .child(list)
            .children(notice.map(|notice| {
                div()
                    .px_4()
                    .py_1()
                    .text_sm()
                    .text_color(rgb(colors.attention))
                    .child(notice)
            }))
            .child(
                div()
                    .px_4()
                    .py_2()
                    .text_xs()
                    .text_color(rgb(colors.text_faint))
                    .child(self.texts.get("settings.apply_hint")),
            )
    }
}
