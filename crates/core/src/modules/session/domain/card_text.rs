//! The text a session shows on its card. Whatever its source, it is one
//! line and bounded: a prompt or a notification can be pages long.

pub const MAX_TITLE_CHARS: usize = 80;
pub const MAX_PREVIEW_CHARS: usize = 200;

const ELLIPSIS: char = '…';

/// The title the user typed; `None` when it leaves nothing to show.
pub fn title_from(typed: &str) -> Option<String> {
    one_line(typed, MAX_TITLE_CHARS)
}

/// One line of a message, as text: Claude writes markdown, and a card
/// shows it raw. `None` when it has no text.
pub fn preview_from(message: &str) -> Option<String> {
    one_line(&without_markdown(message), MAX_PREVIEW_CHARS)
}

/// Drops the basic markers: the `#` of a heading, `**`, `__` and backticks.
/// A lone `*` or `_` is left alone: it is as often a product or a name.
fn without_markdown(text: &str) -> String {
    text.lines()
        .map(|line| {
            let heading = line.trim_start().trim_start_matches('#');
            let line = if heading.starts_with(' ') {
                heading
            } else {
                line
            };
            line.replace("**", "").replace("__", "").replace('`', "")
        })
        .collect::<Vec<_>>()
        .join("\n")
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
