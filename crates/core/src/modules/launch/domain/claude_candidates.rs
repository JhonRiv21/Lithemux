use std::path::{Path, PathBuf};

const TEMPORARY_ROOTS: [&str; 4] = [
    "/tmp",
    "/private/tmp",
    "/var/folders",
    "/private/var/folders",
];

/// Absolute paths printed by `command -v claude` or `which -a claude`, in
/// order and without repeats. Anything else a shell startup file prints is
/// ignored.
pub fn candidate_paths(shell_output: &str) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = Vec::new();
    for line in shell_output.lines().map(str::trim) {
        if line.starts_with('/') && !paths.iter().any(|known| known == Path::new(line)) {
            paths.push(PathBuf::from(line));
        }
    }
    paths
}

/// A `claude` under a temporary directory or a `shims` folder is a wrapper
/// some other tool injected into the PATH, never the user's own install.
pub fn is_injected_wrapper(path: &Path, temp_dir: Option<&Path>) -> bool {
    let in_temp = temp_dir.is_some_and(|temp| path.starts_with(temp))
        || TEMPORARY_ROOTS.iter().any(|root| path.starts_with(root));
    in_temp || path.to_string_lossy().contains("shims")
}
