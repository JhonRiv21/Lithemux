use std::ffi::OsString;
use std::path::PathBuf;

use lithemux_core::modules::launch::{HookBinary, SiblingHookBinary};
use lithemux_core::platform::paths::home::Home;
use lithemux_core::platform::process::shell::user_shell;

use super::error::HostError;

/// What the host takes from the machine it runs on. Tests give it a
/// temporary home and fakes; the app reads the real environment.
#[derive(Debug, Clone)]
pub struct HostConfig {
    pub home: Home,
    /// `LITHEMUX_HOME` to hand on to the sessions, only when it was set.
    pub lithemux_home: Option<PathBuf>,
    pub hook_binary: PathBuf,
    /// The user's login shell: runs shell sessions and looks for `claude`.
    pub shell: PathBuf,
    /// Where Claude Code keeps its files, when it is not the default place.
    pub claude_config_dir: Option<PathBuf>,
    /// The environment the sessions start from. The marks of the session
    /// the app itself may run in are taken out of it.
    pub environment: Vec<(OsString, OsString)>,
}

impl HostConfig {
    pub fn from_environment() -> Result<Self, HostError> {
        Ok(HostConfig {
            home: Home::resolve()?,
            lithemux_home: Home::env_override(),
            hook_binary: SiblingHookBinary.path()?,
            shell: user_shell(),
            claude_config_dir: None,
            environment: std::env::vars_os().collect(),
        })
    }
}
