//! The sessions as the palette searches them. No gpui.

use lithemux_core::modules::category::Category;
use lithemux_core::modules::session::Session;

use super::ranking::Entry;

/// Every session with its category; one whose category is gone is left out.
pub fn entries(categories: &[Category], sessions: &[Session]) -> Vec<Entry> {
    sessions
        .iter()
        .filter_map(|session| {
            let category = categories.iter().find(|c| c.id == session.category_id)?;
            Some(Entry {
                session_id: session.id.clone(),
                title: session.display_title().to_string(),
                slug: session.slug.clone(),
                category: category.name.clone(),
                category_slug: category.slug.clone(),
                role: session.role.clone(),
                last_activity_at: session.last_activity_at,
            })
        })
        .collect()
}
