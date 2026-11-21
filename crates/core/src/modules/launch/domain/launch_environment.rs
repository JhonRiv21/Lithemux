use std::path::PathBuf;

/// What the launcher takes from the process that asks for the launch.
#[derive(Debug, Clone)]
pub struct LaunchEnvironment {
    /// The user's login shell.
    pub shell: PathBuf,
    /// `LITHEMUX_HOME`, only when it is set.
    pub lithemux_home: Option<PathBuf>,
}
