use crate::modules::context::domain::user_text::{cap, one_line};

const MAX_ROLE_CHARS: usize = 160;

/// A role is free text: flatten it to one line, drop the final period (the
/// templates add their own punctuation) and cap its length.
pub fn role_text(role: &str) -> String {
    let flat = one_line(role, usize::MAX);
    cap(flat.trim_end_matches('.'), MAX_ROLE_CHARS)
}
