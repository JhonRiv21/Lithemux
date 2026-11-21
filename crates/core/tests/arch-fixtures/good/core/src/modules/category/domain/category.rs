use std::path::PathBuf;

use crate::modules::category::domain::slug_policy::is_valid_slug;

pub struct Category {
    pub slug: String,
    pub root: PathBuf,
}

impl Category {
    pub fn is_valid(&self) -> bool {
        is_valid_slug(&self.slug)
    }
}
