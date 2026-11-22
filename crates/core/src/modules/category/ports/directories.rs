use std::path::{Path, PathBuf};

use crate::modules::category::error::CategoryError;

pub trait Directories {
    /// The absolute, symlink-free form of `path`. Fails when it is not an
    /// existing directory.
    fn existing(&self, path: &Path) -> Result<PathBuf, CategoryError>;
}
