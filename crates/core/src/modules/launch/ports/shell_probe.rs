use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

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
    /// What `script` printed in the user's shell with stdin closed, without
    /// anything the shell's own startup or exit printed around it. `None`
    /// when it fails or takes longer than `timeout`.
    fn run(&self, mode: ShellMode, script: &str, timeout: Duration) -> Option<String>;
    fn is_executable(&self, path: &Path) -> bool;
    /// Raw output of `<claude> --version`, or `None` on failure or timeout.
    fn version_output(&self, claude_bin: &Path, timeout: Duration) -> Option<String>;
    /// The clock the waits are measured with.
    fn now(&self) -> Instant;
    /// The folders where files are temporary on this machine: a `claude`
    /// found there is something another tool put on the PATH.
    fn temporary_roots(&self) -> Vec<PathBuf>;
}
