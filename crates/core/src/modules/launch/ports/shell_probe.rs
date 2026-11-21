use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShellMode {
    /// `$SHELL -lic`: loads the user's interactive setup, where version
    /// managers and native installers usually extend the PATH.
    LoginInteractive,
    /// `$SHELL -lc`.
    Login,
}

/// Asks the user's shell and file system about `claude`.
pub trait ShellProbe {
    /// Stdout of running `script` in the user's shell with stdin closed, or
    /// `None` when it fails or takes longer than `timeout`.
    fn run(&self, mode: ShellMode, script: &str, timeout: Duration) -> Option<String>;
    fn is_executable(&self, path: &Path) -> bool;
    /// Raw output of `<claude> --version`, or `None` on failure or timeout.
    fn version_output(&self, claude_bin: &Path, timeout: Duration) -> Option<String>;
    /// The user's temporary directory (`$TMPDIR`), when known.
    fn temp_dir(&self) -> Option<PathBuf>;
}
