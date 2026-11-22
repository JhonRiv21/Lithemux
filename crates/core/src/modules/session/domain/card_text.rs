//! The text a session shows on its card. Whatever its source, it is one
//! line and bounded: a prompt or a notification can be pages long.

pub const MAX_TITLE_CHARS: usize = 80;
pub const MAX_PREVIEW_CHARS: usize = 200;

const ELLIPSIS: char = '…';

/// The title the user typed; `None` when it leaves nothing to show.
pub fn title_from(typed: &str) -> Option<String> {
    one_line(typed, MAX_TITLE_CHARS)
}

/// One line of a message; `None` when it has no text.
pub fn preview_from(message: &str) -> Option<String> {
    one_line(message, MAX_PREVIEW_CHARS)
}

fn one_line(text: &str, max_chars: usize) -> Option<String> {
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if flat.is_empty() {
        return None;
    }
    if flat.chars().count() <= max_chars {
        return Some(flat);
    }
    let mut short: String = flat.chars().take(max_chars.saturating_sub(1)).collect();
    short.push(ELLIPSIS);
    Some(short)
}
