use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::modules::launch::ports::shell_probe::{ShellMode, ShellProbe};
use crate::platform::process::capture::stdout_within;

pub struct LoginShellProbe {
    shell: PathBuf,
}

impl LoginShellProbe {
    pub fn new(shell: impl Into<PathBuf>) -> Self {
        LoginShellProbe {
            shell: shell.into(),
        }
    }
}

impl ShellProbe for LoginShellProbe {
    fn run(&self, mode: ShellMode, script: &str, timeout: Duration) -> Option<String> {
        let flags = match mode {
            ShellMode::LoginInteractive => "-lic",
            ShellMode::Login => "-lc",
        };
        stdout_within(&self.shell, &[flags, script], timeout)
    }

    fn is_executable(&self, path: &Path) -> bool {
        path.metadata()
            .is_ok_and(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
    }

    fn version_output(&self, claude_bin: &Path, timeout: Duration) -> Option<String> {
        stdout_within(claude_bin, &["--version"], timeout)
    }

    fn temp_dir(&self) -> Option<PathBuf> {
        std::env::var_os("TMPDIR")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
    }
}
