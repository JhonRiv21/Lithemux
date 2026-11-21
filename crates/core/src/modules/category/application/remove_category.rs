use crate::modules::category::application::find_category::get_category;
use crate::modules::category::domain::category::Category;
use crate::modules::category::error::CategoryError;
use crate::modules::category::ports::category_repository::CategoryRepository;

/// Removes the category and its sessions. The board is user-written content
/// and stays on disk.
pub fn remove_category(
    categories: &dyn CategoryRepository,
    slug: &str,
) -> Result<Category, CategoryError> {
    let category = get_category(categories, slug)?;
    categories.delete(&category.id)?;
    Ok(category)
}
