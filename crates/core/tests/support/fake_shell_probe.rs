use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use lithemux_core::modules::launch::{PathKind, ShellMode, ShellProbe, TEMPORARY_ROOTS};

// Same contract as LoginShellProbe: `None` stands for a failure or a timeout.
// Its clock is made up: a question takes what `taking` says, or nothing when
// it is answered and its whole wait when it is not.
pub struct FakeShellProbe {
    outputs: HashMap<(ShellMode, String), String>,
    executables: Vec<PathBuf>,
    plain_files: Vec<PathBuf>,
    directories: Vec<PathBuf>,
    versions: HashMap<PathBuf, String>,
    temporary_roots: Vec<PathBuf>,
    costs: HashMap<ShellMode, Duration>,
    started: Instant,
    elapsed: Cell<Duration>,
    pub calls: RefCell<Vec<(ShellMode, String, Duration)>>,
}

impl FakeShellProbe {
    pub fn new() -> Self {
        FakeShellProbe {
            outputs: HashMap::new(),
            executables: Vec::new(),
            plain_files: Vec::new(),
            directories: Vec::new(),
            versions: HashMap::new(),
            temporary_roots: TEMPORARY_ROOTS.iter().map(PathBuf::from).collect(),
            costs: HashMap::new(),
            started: Instant::now(),
            elapsed: Cell::new(Duration::ZERO),
            calls: RefCell::new(Vec::new()),
        }
    }

    /// How long every question in that shell mode takes, answered or not.
    pub fn taking(mut self, mode: ShellMode, time: Duration) -> Self {
        self.costs.insert(mode, time);
        self
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

    /// A file that is there but cannot run.
    pub fn plain_file(mut self, path: &str) -> Self {
        self.plain_files.push(PathBuf::from(path));
        self
    }

    pub fn directory(mut self, path: &str) -> Self {
        self.directories.push(PathBuf::from(path));
        self
    }

    pub fn version(mut self, path: &str, output: &str) -> Self {
        self.versions
            .insert(PathBuf::from(path), output.to_string());
        self
    }

    pub fn temp_dir(mut self, path: &str) -> Self {
        self.temporary_roots.push(PathBuf::from(path));
        self
    }
}

impl ShellProbe for FakeShellProbe {
    fn run(&self, mode: ShellMode, script: &str, timeout: Duration) -> Option<String> {
        self.calls
            .borrow_mut()
            .push((mode, script.to_string(), timeout));
        let answer = self.outputs.get(&(mode, script.to_string())).cloned();
        let unanswered = if answer.is_some() {
            Duration::ZERO
        } else {
            timeout
        };
        let took = self.costs.get(&mode).copied().unwrap_or(unanswered);
        self.elapsed.set(self.elapsed.get() + took);
        answer
    }

    fn now(&self) -> Instant {
        self.started + self.elapsed.get()
    }

    fn is_executable(&self, path: &Path) -> bool {
        self.executables.iter().any(|known| known == path)
    }

    fn path_kind(&self, path: &Path) -> PathKind {
        let known = |list: &[PathBuf]| list.iter().any(|known| known == path);
        if known(&self.executables) {
            PathKind::File { executable: true }
        } else if known(&self.plain_files) {
            PathKind::File { executable: false }
        } else if known(&self.directories) {
            PathKind::Directory
        } else {
            PathKind::Missing
        }
    }

    fn version_output(&self, claude_bin: &Path, _timeout: Duration) -> Option<String> {
        self.versions.get(claude_bin).cloned()
    }

    fn temporary_roots(&self) -> Vec<PathBuf> {
        self.temporary_roots.clone()
    }
}
