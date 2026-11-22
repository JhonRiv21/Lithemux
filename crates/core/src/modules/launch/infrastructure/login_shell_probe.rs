use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use uuid::Uuid;

use crate::modules::launch::domain::claude_candidates::TEMPORARY_ROOTS;
use crate::modules::launch::domain::shell_answer::{marked_answer, marked_script};
use crate::modules::launch::ports::shell_probe::{ShellMode, ShellProbe};
use crate::platform::process::capture::stdout_within;

pub struct LoginShellProbe {
    shell: PathBuf,
    temporary_roots: Vec<PathBuf>,
}

impl LoginShellProbe {
    /// With the temporary folders of this machine: the usual ones and
    /// `$TMPDIR`.
    pub fn new(shell: impl Into<PathBuf>) -> Self {
        let mut temporary_roots: Vec<PathBuf> = TEMPORARY_ROOTS.iter().map(PathBuf::from).collect();
        temporary_roots.extend(
            std::env::var_os("TMPDIR")
                .filter(|value| !value.is_empty())
                .map(PathBuf::from),
        );
        Self::with_temporary_roots(shell, temporary_roots)
    }

    /// For a caller that knows which folders are temporary where it runs.
    pub fn with_temporary_roots(shell: impl Into<PathBuf>, temporary_roots: Vec<PathBuf>) -> Self {
        LoginShellProbe {
            shell: shell.into(),
            temporary_roots,
        }
    }
}

impl ShellProbe for LoginShellProbe {
    fn run(&self, mode: ShellMode, script: &str, timeout: Duration) -> Option<String> {
        let flags = match mode {
            ShellMode::LoginInteractive => "-lic",
            ShellMode::Login => "-lc",
        };
        // A marker nobody can guess, so nothing the profile prints passes
        // for the answer.
        let marker = format!("lithemux-{}", Uuid::new_v4().simple());
        let output = stdout_within(
            &self.shell,
            &[flags, &marked_script(script, &marker)],
            timeout,
        )?;
        marked_answer(&output, &marker).map(str::to_string)
    }

    fn now(&self) -> Instant {
        Instant::now()
    }

    fn is_executable(&self, path: &Path) -> bool {
        path.metadata()
            .is_ok_and(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
    }

    fn version_output(&self, claude_bin: &Path, timeout: Duration) -> Option<String> {
        stdout_within(claude_bin, &["--version"], timeout)
    }

    fn temporary_roots(&self) -> Vec<PathBuf> {
        self.temporary_roots.clone()
    }
}
