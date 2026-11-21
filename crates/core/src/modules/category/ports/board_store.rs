use std::path::PathBuf;

use crate::modules::category::domain::category::Category;
use crate::modules::category::error::CategoryError;

/// The board is a Markdown file per category, kept outside any user repo.
pub trait BoardStore {
    fn path(&self, category_slug: &str) -> PathBuf;
    /// Creates the board with a heading when it does not exist yet. An
    /// existing board is never overwritten.
    fn ensure(&self, category: &Category) -> Result<(), CategoryError>;
}
