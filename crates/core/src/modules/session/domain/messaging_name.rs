/// Address of a session in `ListAgents`/`SendMessage`. Computed, never stored.
pub fn messaging_name(category_slug: &str, session_slug: &str) -> String {
    format!("{category_slug}-{session_slug}")
}

/// The `<category>/<session>` that already answers to `name`, if any. Two
/// different pairs can collide: "a-b"/"c" and "a"/"b-c" are both "a-b-c".
pub fn name_owner<'a>(
    name: &str,
    existing: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> Option<String> {
    existing
        .into_iter()
        .find(|(category, session)| messaging_name(category, session) == name)
        .map(|(category, session)| format!("{category}/{session}"))
}
