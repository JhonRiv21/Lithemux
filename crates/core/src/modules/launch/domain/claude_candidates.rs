use std::path::{Path, PathBuf};

/// Where temporary files live on a Mac, whatever `$TMPDIR` says.
pub const TEMPORARY_ROOTS: [&str; 4] = [
    "/tmp",
    "/private/tmp",
    "/var/folders",
    "/private/var/folders",
];

const CLAUDE_FILE_NAME: &str = "claude";

/// Absolute paths to a file called `claude` printed by `command -v claude`
/// or `which -a claude`, in order and without repeats. Anything else a shell
/// startup file prints is ignored, other paths included: whatever answers
/// for `claude` is called `claude`.
pub fn candidate_paths(shell_output: &str) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = Vec::new();
    for line in shell_output.lines().map(str::trim) {
        let path = Path::new(line);
        if line.starts_with('/') && is_called_claude(path) && !paths.iter().any(|p| p == path) {
            paths.push(path.to_path_buf());
        }
    }
    paths
}

pub fn is_called_claude(path: &Path) -> bool {
    path.file_name()
        .is_some_and(|name| name == CLAUDE_FILE_NAME)
}

/// A `claude` under a temporary directory or a `shims` folder is a wrapper
/// some other tool injected into the PATH, never the user's own install.
pub fn is_injected_wrapper(path: &Path, temporary_roots: &[PathBuf]) -> bool {
    let in_temp = temporary_roots.iter().any(|root| path.starts_with(root));
    in_temp || path.to_string_lossy().contains("shims")
}
