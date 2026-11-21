use crate::modules::category::domain::category::Category;

pub trait CategoryRepository {
    fn insert(&self, category: &Category);
}
