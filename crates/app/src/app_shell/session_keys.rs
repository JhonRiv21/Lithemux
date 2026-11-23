//! The keys that go from session to session (PLAN §3.5). Each one is a
//! letter or an arrow, the same key on every keyboard layout (D-38): on a
//! Latin American keyboard a bracket needs Option and never arrives as
//! `cmd-]`.

use gpui::{App, KeyBinding, actions};

actions!(
    lithemux,
    [GoToSession, NextSession, PreviousSession, SessionMenu]
);

pub fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("cmd-p", GoToSession, None),
        KeyBinding::new("cmd-alt-down", NextSession, None),
        KeyBinding::new("cmd-alt-up", PreviousSession, None),
        KeyBinding::new("cmd-shift-m", SessionMenu, None),
    ]);
}
