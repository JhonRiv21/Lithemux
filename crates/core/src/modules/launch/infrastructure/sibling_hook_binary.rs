use std::path::PathBuf;

use crate::modules::launch::domain::hooks_settings::HOOK_BIN_NAME;
use crate::modules::launch::error::LaunchError;
use crate::modules::launch::ports::hook_binary::HookBinary;
use crate::platform::paths::fs_error::FsError;

/// `lithemux-hook` is installed next to the running binary.
pub struct SiblingHookBinary;

impl HookBinary for SiblingHookBinary {
    fn path(&self) -> Result<PathBuf, LaunchError> {
        let exe =
            std::env::current_exe().map_err(|e| FsError::new("locating the current binary", e))?;
        let exe = exe.canonicalize().unwrap_or(exe);
        let dir = exe.parent().ok_or_else(|| {
            FsError::new(
                "locating the current binary",
                std::io::Error::other("the binary has no parent directory"),
            )
        })?;
        Ok(dir.join(HOOK_BIN_NAME))
    }
}
