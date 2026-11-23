use crate::modules::category::domain::latin_fold::fold_latin;

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

/// The slug a name typed by the user gives: lowercase, accents dropped
/// («Migración» is `migracion`), with a hyphen for each run of anything
/// else. `None` when nothing usable is left. Slugs already stored are
/// never recomputed.
pub fn slug_from_name(name: &str) -> Option<String> {
    let mut slug = String::new();
    for character in fold_latin(name.trim()).chars().flat_map(char::to_lowercase) {
        if character.is_ascii_lowercase() || character.is_ascii_digit() {
            slug.push(character);
        } else if !slug.is_empty() && !slug.ends_with('-') {
            slug.push('-');
        }
    }
    slug.truncate(MAX_SLUG_LEN);
    let slug = slug.trim_end_matches('-');
    is_valid_slug(slug).then(|| slug.to_string())
}

/// `base`, or `base-2`, `base-3`… the first one that `is_taken` does not
/// claim. The base is cut to leave room for the number, so a numbered slug
/// is never longer than `MAX_SLUG_LEN`.
pub fn free_slug(base: &str, is_taken: impl Fn(&str) -> bool) -> String {
    if !is_taken(base) {
        return base.to_string();
    }
    let mut number = 2;
    loop {
        let suffix = format!("-{number}");
        let room = MAX_SLUG_LEN.saturating_sub(suffix.len());
        let cut: String = base.chars().take(room).collect();
        let candidate = format!("{}{suffix}", cut.trim_end_matches('-'));
        if !is_taken(&candidate) {
            return candidate;
        }
        number += 1;
    }
}
