use crate::modules::category::domain::category::Category;
use crate::modules::category::error::CategoryError;

pub trait CategoryRepository {
    fn insert(&self, category: &Category) -> Result<(), CategoryError>;
    fn next_sort(&self) -> Result<i64, CategoryError>;
    /// Ordered by `sort`, then creation time.
    fn list(&self) -> Result<Vec<Category>, CategoryError>;
    fn find_by_slug(&self, slug: &str) -> Result<Option<Category>, CategoryError>;
    fn find_by_id(&self, id: &str) -> Result<Option<Category>, CategoryError>;
    fn update_name(&self, id: &str, name: &str) -> Result<(), CategoryError>;
    fn update_sort(&self, id: &str, sort: i64) -> Result<(), CategoryError>;
    fn update_collapsed(&self, id: &str, collapsed: bool) -> Result<(), CategoryError>;
    /// Also removes the sessions that belong to the category.
    fn delete(&self, id: &str) -> Result<(), CategoryError>;
}
