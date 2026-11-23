//! What the main window does with the palette and the keys of the
//! sessions: opens it, jumps where it says, and steps through the sidebar.

use std::collections::HashMap;
use std::rc::Rc;

use gpui::{AppContext, Context, Div, Window, prelude::*};

use super::session_keys::{GoToSession, NextSession, PreviousSession, SessionMenu};
use super::window::MainWindow;
use crate::palette::entries::entries;
use crate::palette::{Palette, PaletteEvent};
use crate::sidebar::keyboard_order::{step, visible_order};
use crate::sidebar::status::status_of;
use crate::sidebar::status_text::status_text;

impl MainWindow {
    pub(super) fn open_palette(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.palette.is_some() {
            return;
        }
        let live = self.live.read(cx);
        let found = entries(live.categories(), live.sessions());
        let texts = Rc::clone(&live.texts);
        let statuses: HashMap<String, String> = live
            .sessions()
            .iter()
            .map(|session| {
                let phase = live.shown_phase(&session.id);
                let status = status_of(phase.as_ref(), session.state, session.profile);
                (session.id.clone(), status_text(&status, &texts))
            })
            .collect();
        let palette = cx.new(|cx| Palette::new(found, statuses, texts, window, cx));
        cx.subscribe_in(&palette, window, |this, _, event, window, cx| {
            this.palette = None;
            match event {
                PaletteEvent::Jump(id) => this.show_session(id, window, cx),
                PaletteEvent::Close => this.workspace.update(cx, |workspace, cx| {
                    workspace.focus_what_is_shown(window, cx)
                }),
            }
            cx.notify();
        })
        .detach();
        self.palette = Some(palette);
        cx.notify();
    }

    /// The same as a click on its card: shown, and woken if it hibernates.
    fn show_session(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.workspace
            .update(cx, |workspace, cx| workspace.show_session(id, window, cx));
    }

    fn step_session(&mut self, by: isize, window: &mut Window, cx: &mut Context<Self>) {
        let order = visible_order(&self.live.read(cx).tree());
        let current = self
            .workspace
            .read(cx)
            .session_in_focus()
            .map(str::to_string);
        if let Some(id) = step(&order, current.as_deref(), by) {
            self.show_session(&id, window, cx);
        }
    }

    fn open_the_session_menu(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(id) = self
            .workspace
            .read(cx)
            .session_in_focus()
            .map(str::to_string)
        else {
            return;
        };
        self.sidebar
            .update(cx, |sidebar, cx| sidebar.open_session_menu(&id, window, cx));
    }

    pub(super) fn handle_session_keys(&self, root: Div, cx: &mut Context<Self>) -> Div {
        root.on_action(cx.listener(|this, _: &GoToSession, window, cx| {
            this.open_palette(window, cx);
        }))
        .on_action(cx.listener(|this, _: &NextSession, window, cx| {
            this.step_session(1, window, cx);
        }))
        .on_action(cx.listener(|this, _: &PreviousSession, window, cx| {
            this.step_session(-1, window, cx);
        }))
        .on_action(cx.listener(|this, _: &SessionMenu, window, cx| {
            this.open_the_session_menu(window, cx);
        }))
    }
}
