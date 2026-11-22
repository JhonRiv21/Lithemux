use std::path::Path;

use crate::modules::category::error::CategoryError;
use crate::modules::category::ports::directories::Directories;

/// The directory a category or a session works in, as it is stored: an
/// absolute path that exists when it is registered.
pub fn locate_directory(
    directories: &dyn Directories,
    path: &Path,
) -> Result<String, CategoryError> {
    Ok(directories.existing(path)?.display().to_string())
}
