//! Why a `claude_path` setting cannot run, said the way the user can fix
//! it. Pure: what is on disk comes in as a `PathKind`.

use std::fmt;
use std::path::Path;

/// What there is at a path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathKind {
    Missing,
    Directory,
    File { executable: bool },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaudePathProblem {
    /// `"/path"`: the settings are kept without quotes.
    Quoted,
    NotAbsolute,
    Missing,
    Directory,
    NotExecutable,
}

impl fmt::Display for ClaudePathProblem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            ClaudePathProblem::Quoted => "it has quotes around it",
            ClaudePathProblem::NotAbsolute => "it is not an absolute path",
            ClaudePathProblem::Missing => "nothing is there",
            ClaudePathProblem::Directory => "it is a folder, not a program",
            ClaudePathProblem::NotExecutable => "the file is not executable",
        })
    }
}

/// What keeps `value` from running, if anything: first what is wrong with
/// how it is written, then what `kind_of` finds on disk.
pub fn claude_path_problem(
    value: &str,
    kind_of: impl FnOnce(&Path) -> PathKind,
) -> Option<ClaudePathProblem> {
    let quotes = ['"', '\''];
    if value.starts_with(quotes) || value.ends_with(quotes) {
        return Some(ClaudePathProblem::Quoted);
    }
    if !value.starts_with('/') {
        return Some(ClaudePathProblem::NotAbsolute);
    }
    match kind_of(Path::new(value)) {
        PathKind::Missing => Some(ClaudePathProblem::Missing),
        PathKind::Directory => Some(ClaudePathProblem::Directory),
        PathKind::File { executable: false } => Some(ClaudePathProblem::NotExecutable),
        PathKind::File { executable: true } => None,
    }
}
