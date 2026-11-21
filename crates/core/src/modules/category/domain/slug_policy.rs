pub const MAX_SLUG_LEN: usize = 48;

/// `[a-z0-9-]`, not empty. A leading or trailing hyphen is rejected too: the
/// slug ends up in `claude -n <name>`, where `-foo` would read as a flag.
pub fn is_valid_slug(value: &str) -> bool {
    let charset_ok = value
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
    !value.is_empty()
        && value.len() <= MAX_SLUG_LEN
        && charset_ok
        && !value.starts_with('-')
        && !value.ends_with('-')
}
