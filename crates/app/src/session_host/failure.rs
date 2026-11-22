//! Why a session could not start, as something the interface can put in
//! words of its own language.

use lithemux_core::modules::launch::LaunchError;

use super::error::HostError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Failure {
    /// `claude` is nowhere to be found; with what the lookup said.
    ClaudeNotFound(String),
    /// The session's working directory is gone.
    MissingFolder(String),
    /// `lithemux-hook` is not next to the app.
    HookMissing(String),
    /// The program was found and did not start.
    CouldNotStart(String),
    /// Anything else, with its technical detail.
    Other(String),
}

impl From<&HostError> for Failure {
    fn from(error: &HostError) -> Self {
        match error {
            HostError::Launch(LaunchError::ClaudeNotFound(detail)) => {
                Failure::ClaudeNotFound(detail.clone())
            }
            HostError::Launch(LaunchError::MissingCwd(folder)) => {
                Failure::MissingFolder(folder.display().to_string())
            }
            HostError::Launch(LaunchError::HookNotExecutable(hook)) => {
                Failure::HookMissing(hook.display().to_string())
            }
            HostError::Launch(LaunchError::Spawn { program, .. }) => {
                Failure::CouldNotStart(program.display().to_string())
            }
            other => Failure::Other(other.to_string()),
        }
    }
}
