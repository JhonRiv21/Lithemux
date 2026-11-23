use std::io;
use std::path::PathBuf;

use crate::modules::launch::domain::claude_path_policy::ClaudePathProblem;
use crate::modules::session::SessionError;
use crate::modules::settings::SettingsError;
use crate::platform::paths::fs_error::FsError;

#[derive(Debug, thiserror::Error)]
pub enum LaunchError {
    #[error("the session is already open{}", .pid.map(|p| format!(" (pid {p})")).unwrap_or_default())]
    AlreadyOpen { pid: Option<u32> },

    #[error("could not find `claude`: {0}")]
    ClaudeNotFound(String),

    #[error(
        "the claude_path setting is {value:?} and {problem}; fix it, or leave it empty to use the claude of your PATH (until then, no claude is run)"
    )]
    ClaudePathUnusable {
        value: String,
        problem: ClaudePathProblem,
    },

    #[error(
        "{0} is missing or not executable; reinstall lithemux so that lithemux-hook sits next to this binary"
    )]
    HookNotExecutable(PathBuf),

    #[error("session \"{0}\" has no Claude session id")]
    MissingClaudeSessionId(String),

    #[error("the working directory {0} does not exist")]
    MissingCwd(PathBuf),

    #[error("could not start {program}: {source}")]
    Spawn {
        program: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error(transparent)]
    Fs(#[from] FsError),

    #[error(transparent)]
    Session(#[from] SessionError),

    #[error(transparent)]
    Settings(#[from] SettingsError),
}
