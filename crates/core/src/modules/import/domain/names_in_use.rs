use std::collections::HashSet;
use std::path::Path;

use crate::modules::category::{Category, free_slug, slug_from_name};
use crate::modules::session::{Session, messaging_name};

/// When nothing usable is left of a name.
const SESSION_FALLBACK: &str = "session";
const CATEGORY_FALLBACK: &str = "imported";

/// The slugs and messaging names already taken, so that what is imported
/// never collides with what exists, nor with itself.
pub struct NamesInUse {
    categories: HashSet<String>,
    sessions: HashSet<(String, String)>,
    messaging: HashSet<String>,
}

impl NamesInUse {
    pub fn of(categories: &[Category], sessions: &[Session]) -> Self {
        let slug_of = |id: &str| {
            categories
                .iter()
                .find(|category| category.id == id)
                .map(|category| category.slug.clone())
        };
        let pairs: Vec<(String, String)> = sessions
            .iter()
            .filter_map(|session| Some((slug_of(&session.category_id)?, session.slug.clone())))
            .collect();
        NamesInUse {
            categories: categories.iter().map(|c| c.slug.clone()).collect(),
            messaging: pairs.iter().map(|(c, s)| messaging_name(c, s)).collect(),
            sessions: pairs.into_iter().collect(),
        }
    }

    /// The name and slug of a new category for `folder`, named after it.
    pub fn claim_category(&mut self, folder: &str) -> (String, String) {
        let name = Path::new(folder)
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| CATEGORY_FALLBACK.to_string());
        let base = slug_from_name(&name).unwrap_or_else(|| CATEGORY_FALLBACK.to_string());
        let slug = free_slug(&base, |slug| self.categories.contains(slug));
        self.categories.insert(slug.clone());
        (name, slug)
    }

    /// The slug of a session called `name` in the category `category`.
    pub fn claim_session(&mut self, category: &str, name: &str) -> String {
        let base = slug_from_name(name).unwrap_or_else(|| SESSION_FALLBACK.to_string());
        let slug = free_slug(&base, |slug| {
            self.sessions
                .contains(&(category.to_string(), slug.to_string()))
                || self.messaging.contains(&messaging_name(category, slug))
        });
        self.messaging.insert(messaging_name(category, &slug));
        self.sessions.insert((category.to_string(), slug.clone()));
        slug
    }
}
