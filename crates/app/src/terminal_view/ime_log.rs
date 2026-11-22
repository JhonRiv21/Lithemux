//! Diagnostic trace of what the input method and the keyboard deliver, for
//! bugs that only show with a real keyboard. Off unless `LITHEMUX_IME_LOG=1`.
//! It never prints what was typed: only which event came, whether something
//! was being composed, and lengths.

use std::ffi::OsStr;
use std::fmt::Arguments;
use std::sync::OnceLock;
use std::time::Instant;

const SWITCH: &str = "LITHEMUX_IME_LOG";
const ON: &str = "1";

fn started() -> Option<Instant> {
    static STARTED: OnceLock<Option<Instant>> = OnceLock::new();
    *STARTED.get_or_init(|| is_switched_on(std::env::var_os(SWITCH).as_deref()).then(Instant::now))
}

/// Only the exact value `1` turns the trace on.
pub(super) fn is_switched_on(value: Option<&OsStr>) -> bool {
    value.is_some_and(|value| value == ON)
}

/// One line on stderr, stamped with the milliseconds since the first one.
pub(super) fn trace(event: Arguments<'_>) {
    if let Some(started) = started() {
        eprintln!("ime {:>7} ms  {event}", started.elapsed().as_millis());
    }
}

/// The size of a text without the text: characters and UTF-8 bytes. An
/// accented letter is one character in two bytes; a dead key followed by a
/// plain letter arrives as two texts of one character each.
pub(super) fn size_of(text: &str) -> String {
    format!("{} chars / {} bytes", text.chars().count(), text.len())
}

/// What kind of key it was, never which one. A key that produces no text
/// is a dead key only when nothing is held with it: with control or command
/// it is a shortcut.
pub(super) fn kind_of_key(key: &str, produces_text: bool, with_modifier: bool) -> &'static str {
    match (key.chars().count(), produces_text) {
        (1, true) => "character",
        (1, false) if with_modifier => "modified key",
        (1, false) => "dead key",
        _ => "named key",
    }
}
