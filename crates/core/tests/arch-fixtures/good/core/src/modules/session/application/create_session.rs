use crate::modules::category::{Category, CategoryRepository};
use crate::modules::session::domain::session::Session;

pub fn create_session(categories: &dyn CategoryRepository, category: &Category) -> Session {
    categories.insert(category);
    Session {
        slug: String::new(),
    }
}
