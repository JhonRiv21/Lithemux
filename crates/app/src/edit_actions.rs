//! Copy and paste as actions, so that the Edit menu and the keyboard ask
//! for the same thing and whatever has the focus answers.

use gpui::{App, KeyBinding, actions};

actions!(edit, [Copy, Paste]);

pub fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("cmd-c", Copy, None),
        KeyBinding::new("cmd-v", Paste, None),
    ]);
}
