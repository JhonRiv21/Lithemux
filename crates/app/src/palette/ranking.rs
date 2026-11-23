//! Which sessions a search finds, best first. No gpui.
//!
//! A session is found by its name, slug, category (name or slug) and role,
//! without minding case or accents. A field equal to the search is an exact
//! match; one that starts with it, or has a word that does, a prefix; one
//! with the letters of the search in order, a fuzzy match (across the
//! category and the name too, so «sel rev» finds «Revisión» of Selektal).
//! Exact before prefix before fuzzy; among equals, the most recently active.

use std::cmp::Reverse;

use lithemux_core::modules::category::fold_latin;

/// What a session can be found by.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub session_id: String,
    pub title: String,
    pub slug: String,
    pub category: String,
    pub category_slug: String,
    pub role: String,
    pub last_activity_at: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Fit {
    Exact,
    Prefix,
    Fuzzy,
}

/// The sessions to search, with their text folded once: what is typed is
/// searched for again at every key.
pub struct Index {
    prepared: Vec<Prepared>,
}

struct Prepared {
    entry: Entry,
    /// Title, slug, category, category slug and role, folded.
    fields: [String; 5],
    /// The category and the title together.
    across: String,
}

impl Index {
    pub fn new(entries: Vec<Entry>) -> Self {
        let prepared = entries
            .into_iter()
            .map(|entry| {
                let fields = [
                    &entry.title,
                    &entry.slug,
                    &entry.category,
                    &entry.category_slug,
                    &entry.role,
                ]
                .map(|field| folded(field));
                let across = format!("{} {}", fields[2], fields[0]);
                Prepared {
                    entry,
                    fields,
                    across,
                }
            })
            .collect();
        Index { prepared }
    }

    /// The entries that `query` finds, best first.
    pub fn rank(&self, query: &str) -> Vec<&Entry> {
        let query = folded(query.trim());
        let mut found: Vec<(Fit, &Prepared)> = self
            .prepared
            .iter()
            .filter_map(|prepared| Some((fit_of(&query, prepared)?, prepared)))
            .collect();
        found.sort_by(|(fit, a), (other, b)| {
            fit.cmp(other)
                .then(Reverse(a.entry.last_activity_at).cmp(&Reverse(b.entry.last_activity_at)))
                .then_with(|| a.fields[0].cmp(&b.fields[0]))
        });
        found
            .into_iter()
            .map(|(_, prepared)| &prepared.entry)
            .collect()
    }
}

fn fit_of(query: &str, prepared: &Prepared) -> Option<Fit> {
    if query.is_empty() {
        return Some(Fit::Exact);
    }
    let best = prepared
        .fields
        .iter()
        .filter_map(|field| fit_in(query, field))
        .min();
    best.or_else(|| is_in_order(query, &prepared.across).then_some(Fit::Fuzzy))
}

fn fit_in(query: &str, field: &str) -> Option<Fit> {
    if field == query {
        return Some(Fit::Exact);
    }
    let a_word_starts_with = field
        .split(|c: char| !c.is_alphanumeric())
        .any(|word| word.starts_with(query));
    if field.starts_with(query) || a_word_starts_with {
        return Some(Fit::Prefix);
    }
    is_in_order(query, field).then_some(Fit::Fuzzy)
}

/// The letters of `query`, spaces aside, appear in `field` in that order.
fn is_in_order(query: &str, field: &str) -> bool {
    let mut field = field.chars();
    query
        .chars()
        .filter(|c| !c.is_whitespace())
        .all(|wanted| field.any(|c| c == wanted))
}

/// Lowercase and without accents, written with their letter or after it
/// (`core`'s folding, the same that makes slugs).
fn folded(text: &str) -> String {
    fold_latin(text).to_lowercase()
}
