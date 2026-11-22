use std::path::{Path, PathBuf};

use crate::modules::category::error::CategoryError;
use crate::modules::category::ports::directories::Directories;

pub struct FsDirectories;

impl Directories for FsDirectories {
    fn existing(&self, path: &Path) -> Result<PathBuf, CategoryError> {
        let absolute = path
            .canonicalize()
            .map_err(|_| CategoryError::NotADirectory(path.to_path_buf()))?;
        if !absolute.is_dir() {
            return Err(CategoryError::NotADirectory(absolute));
        }
        Ok(absolute)
    }
}
