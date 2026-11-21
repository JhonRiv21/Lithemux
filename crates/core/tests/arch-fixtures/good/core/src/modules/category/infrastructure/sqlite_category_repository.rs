use std::fs;

use crate::modules::category::domain::category::Category;
use crate::modules::category::ports::category_repository::CategoryRepository;
use crate::platform::store::connection::Db;

pub struct SqliteCategoryRepository<'a> {
    pub db: &'a Db,
}

impl CategoryRepository for SqliteCategoryRepository<'_> {
    fn insert(&self, category: &Category) {
        let _ = fs::metadata(&category.root);
    }
}
