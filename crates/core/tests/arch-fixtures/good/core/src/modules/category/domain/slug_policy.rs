// Pure: no std::fs, no rusqlite.
pub fn is_valid_slug(value: &str) -> bool {
    !value.is_empty()
}
