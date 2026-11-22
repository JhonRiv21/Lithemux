use std::path::PathBuf;

use crate::modules::launch::error::LaunchError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaudeInstallation {
    pub path: PathBuf,
    pub version: Option<String>,
    /// Whether a launch would run this one.
    pub chosen: bool,
    /// Usable as a launch target (an executable that is not an injected wrapper).
    pub usable: bool,
}

#[derive(Debug)]
pub struct ClaudeInspection {
    pub chosen: Result<PathBuf, LaunchError>,
    pub warning: Option<String>,
    pub installations: Vec<ClaudeInstallation>,
    /// The `claude` the interactive shell reported last, kept for the day
    /// that shell does not answer.
    pub remembered: Option<PathBuf>,
    /// More than one usable install, on different versions.
    pub version_mismatch: bool,
}
