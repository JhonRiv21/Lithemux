use crate::modules::category::Category;

pub struct Session {
    pub slug: String,
}

impl Session {
    pub fn name(&self, category: &Category) -> String {
        format!("{}-{}", category.slug, self.slug)
    }
}
