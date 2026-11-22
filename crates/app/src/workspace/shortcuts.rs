//! The keyboard of the workspace. The keys are those of iTerm and cmux
//! (D-38 in docs/decisiones.md).

use gpui::{App, Context, Div, KeyBinding, actions, prelude::*};
use lithemux_core::modules::workspace::Axis;

use super::view::Workspace;

pub(super) const CONTEXT: &str = "Workspace";

actions!(
    workspace,
    [
        NewTerminal,
        SplitRight,
        SplitDown,
        CloseTab,
        ClosePane,
        NextPane,
        PreviousPane,
        NextTab,
        PreviousTab,
    ]
);

pub fn bind_keys(cx: &mut App) {
    let context = Some(CONTEXT);
    cx.bind_keys([
        KeyBinding::new("cmd-t", NewTerminal, context),
        KeyBinding::new("cmd-d", SplitRight, context),
        KeyBinding::new("cmd-shift-d", SplitDown, context),
        KeyBinding::new("cmd-w", CloseTab, context),
        KeyBinding::new("cmd-shift-w", ClosePane, context),
        KeyBinding::new("cmd-]", NextPane, context),
        KeyBinding::new("cmd-[", PreviousPane, context),
        KeyBinding::new("cmd-alt-right", NextPane, context),
        KeyBinding::new("cmd-alt-left", PreviousPane, context),
        KeyBinding::new("cmd-shift-]", NextTab, context),
        KeyBinding::new("cmd-shift-[", PreviousTab, context),
    ]);
}

pub(super) fn handle(root: Div, cx: &mut Context<Workspace>) -> Div {
    root.on_action(cx.listener(|this, _: &NewTerminal, window, cx| {
        this.new_terminal(this.layout.active_pane().id, window, cx);
    }))
    .on_action(cx.listener(|this, _: &SplitRight, window, cx| {
        this.split(this.layout.active_pane().id, Axis::Horizontal, window, cx);
    }))
    .on_action(cx.listener(|this, _: &SplitDown, window, cx| {
        this.split(this.layout.active_pane().id, Axis::Vertical, window, cx);
    }))
    .on_action(cx.listener(|this, _: &CloseTab, window, cx| {
        if let Some(session) = this.session_in_focus().map(str::to_string) {
            this.close_tab(&session, window, cx);
        }
    }))
    .on_action(cx.listener(|this, _: &ClosePane, window, cx| {
        this.close_pane(this.layout.active_pane().id, window, cx);
    }))
    .on_action(cx.listener(|this, _: &NextPane, window, cx| {
        this.layout.focus_next_pane();
        this.changed(window, cx);
    }))
    .on_action(cx.listener(|this, _: &PreviousPane, window, cx| {
        this.layout.focus_previous_pane();
        this.changed(window, cx);
    }))
    .on_action(cx.listener(|this, _: &NextTab, window, cx| {
        this.layout.next_tab();
        this.changed(window, cx);
    }))
    .on_action(cx.listener(|this, _: &PreviousTab, window, cx| {
        this.layout.previous_tab();
        this.changed(window, cx);
    }))
}
