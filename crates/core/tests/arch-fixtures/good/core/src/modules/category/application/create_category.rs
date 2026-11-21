use crate::modules::category::domain::category::Category;
use crate::modules::category::ports::category_repository::CategoryRepository;

pub fn create_category(repo: &dyn CategoryRepository, category: &Category) {
    repo.insert(category);
}
