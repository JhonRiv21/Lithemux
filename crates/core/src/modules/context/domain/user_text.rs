//! Text the user typed (names, roles, paths) on its way into the context.
//! It is always one line: a line break would let it add sentences of its
//! own to what Claude reads as facts from lithemux.

const ELLIPSIS: char = '…';

/// One line, at most `max_chars` long.
pub fn one_line(text: &str, max_chars: usize) -> String {
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    cap(&flat, max_chars)
}

/// Cuts at `max_chars`, marking the cut.
pub fn cap(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let mut short: String = text.chars().take(max_chars.saturating_sub(1)).collect();
    short.push(ELLIPSIS);
    short
}
