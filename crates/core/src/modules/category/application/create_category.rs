use uuid::Uuid;

use crate::modules::category::domain::category::{Category, NewCategory};
use crate::modules::category::error::CategoryError;
use crate::modules::category::ports::board_store::BoardStore;
use crate::modules::category::ports::category_repository::CategoryRepository;

/// Creates the category and its empty board.
pub fn create_category(
    categories: &dyn CategoryRepository,
    boards: &dyn BoardStore,
    new: NewCategory,
    now: i64,
) -> Result<Category, CategoryError> {
    if categories.find_by_slug(new.slug.as_str())?.is_some() {
        return Err(CategoryError::AlreadyExists(new.slug.to_string()));
    }
    let category = Category {
        id: Uuid::new_v4().to_string(),
        slug: new.slug.to_string(),
        name: new.name,
        root_dir: new.root_dir,
        color: new.color,
        sort: categories.next_sort()?,
        created_at: now,
        collapsed: false,
        notify: true,
    };
    categories.insert(&category)?;
    boards.ensure(&category)?;
    Ok(category)
}
