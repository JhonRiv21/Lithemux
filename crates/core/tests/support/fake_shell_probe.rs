use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use lithemux_core::modules::launch::{ShellMode, ShellProbe};

// Same contract as LoginShellProbe: `None` stands for a failure or a timeout.
#[derive(Default)]
pub struct FakeShellProbe {
    outputs: HashMap<(ShellMode, String), String>,
    executables: Vec<PathBuf>,
    versions: HashMap<PathBuf, String>,
    temp_dir: Option<PathBuf>,
    pub calls: RefCell<Vec<(ShellMode, String, Duration)>>,
}

impl FakeShellProbe {
    pub fn new() -> Self {
        Self::default()
    }

    /// What `script` prints in that shell mode. A mode without output behaves
    /// like a shell that failed or timed out.
    pub fn answers(mut self, mode: ShellMode, script: &str, stdout: &str) -> Self {
        self.outputs
            .insert((mode, script.to_string()), stdout.to_string());
        self
    }

    pub fn executable(mut self, path: &str) -> Self {
        self.executables.push(PathBuf::from(path));
        self
    }

    pub fn version(mut self, path: &str, output: &str) -> Self {
        self.versions
            .insert(PathBuf::from(path), output.to_string());
        self
    }

    pub fn temp_dir(mut self, path: &str) -> Self {
        self.temp_dir = Some(PathBuf::from(path));
        self
    }
}

impl ShellProbe for FakeShellProbe {
    fn run(&self, mode: ShellMode, script: &str, timeout: Duration) -> Option<String> {
        self.calls
            .borrow_mut()
            .push((mode, script.to_string(), timeout));
        self.outputs.get(&(mode, script.to_string())).cloned()
    }

    fn is_executable(&self, path: &Path) -> bool {
        self.executables.iter().any(|known| known == path)
    }

    fn version_output(&self, claude_bin: &Path, _timeout: Duration) -> Option<String> {
        self.versions.get(claude_bin).cloned()
    }

    fn temp_dir(&self) -> Option<PathBuf> {
        self.temp_dir.clone()
    }
}
