use std::path::PathBuf;

use lithemux_core::modules::launch::LaunchError;
use lithemux_core::modules::session::SessionError;
use lithemux_core::platform::paths::error::PathsError;
use lithemux_core::platform::store::error::StoreError;

use crate::terminal_view::TerminalError;

#[derive(Debug, thiserror::Error)]
pub enum HostError {
    #[error("lithemux is already running{}", .pid.map(|p| format!(" (pid {p})")).unwrap_or_default())]
    AlreadyRunning { pid: Option<u32> },

    #[error("another program is answering on {0}")]
    SocketInUse(PathBuf),

    #[error("{what} {}: {source}", .path.display())]
    Io {
        what: &'static str,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error(transparent)]
    Paths(#[from] PathsError),

    #[error(transparent)]
    Store(#[from] StoreError),

    #[error(transparent)]
    Session(#[from] SessionError),

    #[error(transparent)]
    Launch(#[from] LaunchError),

    #[error(transparent)]
    Terminal(#[from] TerminalError),
}

impl HostError {
    pub(super) fn io(
        what: &'static str,
        path: impl Into<PathBuf>,
    ) -> impl FnOnce(std::io::Error) -> Self {
        let path = path.into();
        move |source| HostError::Io { what, path, source }
    }
}
