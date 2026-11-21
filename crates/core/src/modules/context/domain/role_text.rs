const MAX_ROLE_CHARS: usize = 160;

/// A role is free text: flatten it to one line, drop the final period (the
/// templates add their own punctuation) and cap its length.
pub fn role_text(role: &str) -> String {
    let flat = role.split_whitespace().collect::<Vec<_>>().join(" ");
    let trimmed = flat.trim_end_matches('.');
    if trimmed.chars().count() <= MAX_ROLE_CHARS {
        return trimmed.to_string();
    }
    let mut short: String = trimmed.chars().take(MAX_ROLE_CHARS - 1).collect();
    short.push('…');
    short
}
