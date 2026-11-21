use crate::modules::category::domain::category::Category;
use crate::modules::category::error::CategoryError;
use crate::modules::category::ports::category_repository::CategoryRepository;

pub fn list_categories(
    categories: &dyn CategoryRepository,
) -> Result<Vec<Category>, CategoryError> {
    categories.list()
}

pub fn get_category(
    categories: &dyn CategoryRepository,
    slug: &str,
) -> Result<Category, CategoryError> {
    categories
        .find_by_slug(slug)?
        .ok_or_else(|| CategoryError::NotFound(slug.to_string()))
}

pub fn find_category_by_id(
    categories: &dyn CategoryRepository,
    id: &str,
) -> Result<Option<Category>, CategoryError> {
    categories.find_by_id(id)
}
