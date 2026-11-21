use std::path::PathBuf;

use crate::modules::launch::error::LaunchError;

pub trait HookBinary {
    /// Absolute path of `lithemux-hook`.
    fn path(&self) -> Result<PathBuf, LaunchError>;
}
