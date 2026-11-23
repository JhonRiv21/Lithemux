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
        ShowTab1,
        ShowTab2,
        ShowTab3,
        ShowTab4,
        ShowTab5,
        ShowTab6,
        ShowTab7,
        ShowTab8,
        ShowTab9,
    ]
);

/// Every shortcut of the workspace. Each action has at least one that is
/// the same key on every keyboard layout (letters, digits, arrows).
///
/// gpui names a key by the character it gives: with shift on a key that is
/// not a letter, macOS reports the shifted character and gpui drops the
/// shift. So «cmd+shift+]» arrives as `cmd-}`, and only on a layout where
/// that key gives `}`; on a Latin American one, where `[` and `]` need
/// Option, it never does. The bracket shortcuts stay for US keyboards.
pub fn bindings() -> Vec<KeyBinding> {
    let context = Some(CONTEXT);
    vec![
        KeyBinding::new("cmd-t", NewTerminal, context),
        KeyBinding::new("cmd-d", SplitRight, context),
        KeyBinding::new("cmd-shift-d", SplitDown, context),
        KeyBinding::new("cmd-w", CloseTab, context),
        KeyBinding::new("cmd-shift-w", ClosePane, context),
        KeyBinding::new("cmd-alt-right", NextPane, context),
        KeyBinding::new("cmd-alt-left", PreviousPane, context),
        KeyBinding::new("cmd-]", NextPane, context),
        KeyBinding::new("cmd-[", PreviousPane, context),
        KeyBinding::new("cmd-shift-right", NextTab, context),
        KeyBinding::new("cmd-shift-left", PreviousTab, context),
        KeyBinding::new("cmd-}", NextTab, context),
        KeyBinding::new("cmd-{", PreviousTab, context),
        KeyBinding::new("cmd-1", ShowTab1, context),
        KeyBinding::new("cmd-2", ShowTab2, context),
        KeyBinding::new("cmd-3", ShowTab3, context),
        KeyBinding::new("cmd-4", ShowTab4, context),
        KeyBinding::new("cmd-5", ShowTab5, context),
        KeyBinding::new("cmd-6", ShowTab6, context),
        KeyBinding::new("cmd-7", ShowTab7, context),
        KeyBinding::new("cmd-8", ShowTab8, context),
        KeyBinding::new("cmd-9", ShowTab9, context),
    ]
}

pub fn bind_keys(cx: &mut App) {
    cx.bind_keys(bindings());
}

/// `ShowTab1`…`ShowTab9`, each showing the tab at its place.
macro_rules! show_tab_at {
    ($root:expr, $cx:expr, $($action:ident => $index:expr),+) => {
        $root$(.on_action($cx.listener(|this, _: &$action, window, cx| {
            this.layout.show_tab_at($index);
            this.changed(window, cx);
        })))+
    };
}

pub(super) fn handle(root: Div, cx: &mut Context<Workspace>) -> Div {
    let root = root
        .on_action(cx.listener(|this, _: &NewTerminal, window, cx| {
            this.new_terminal(this.layout.active_pane().id, window, cx);
        }))
        .on_action(cx.listener(|this, _: &SplitRight, window, cx| {
            this.split(this.layout.active_pane().id, Axis::Horizontal, window, cx);
        }))
        .on_action(cx.listener(|this, _: &SplitDown, window, cx| {
            this.split(this.layout.active_pane().id, Axis::Vertical, window, cx);
        }))
        .on_action(cx.listener(|this, _: &CloseTab, window, cx| {
            #[cfg(all(feature = "webview-spike", target_os = "macos"))]
            if this.shows_browser(this.layout.active_pane().id) {
                this.close_browser(this.layout.active_pane().id, window, cx);
                return;
            }
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
        }));
    show_tab_at!(
        root, cx,
        ShowTab1 => 0, ShowTab2 => 1, ShowTab3 => 2, ShowTab4 => 3, ShowTab5 => 4,
        ShowTab6 => 5, ShowTab7 => 6, ShowTab8 => 7, ShowTab9 => 8
    )
}
