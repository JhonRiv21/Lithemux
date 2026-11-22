//! The environment a program gets inside a terminal.

use std::ffi::OsString;

use lithemux_core::platform::process::session_markers::without_session_markers;

use super::pty::ShellCommand;

/// What every program is told about the terminal it runs in.
const TERMINAL: [(&str, &str); 3] = [
    ("TERM", "xterm-256color"),
    ("COLORTERM", "truecolor"),
    ("TERM_PROGRAM", "lithemux"),
];

/// `inherited` (this process's environment) without the marks of the session
/// lithemux itself may be running in, then the terminal's own variables, then
/// what the command asks for. A `claude` that inherits those marks believes
/// it is nested and does not save its transcript.
pub(super) fn child_env(
    command: &ShellCommand,
    inherited: impl Iterator<Item = (OsString, OsString)>,
) -> Vec<(OsString, OsString)> {
    let mut env = Vec::new();
    if command.inherit_env {
        env = without_session_markers(inherited);
    }
    let own = TERMINAL
        .iter()
        .map(|(name, value)| (*name, *value))
        .chain(command.env.iter().map(|(name, value)| (&**name, &**value)));
    for (name, value) in own {
        env.retain(|(existing, _)| existing != name);
        env.push((name.into(), value.into()));
    }
    env
}
