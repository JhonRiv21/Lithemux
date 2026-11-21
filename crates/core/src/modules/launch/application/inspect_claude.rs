use std::path::PathBuf;
use std::time::Duration;

use crate::modules::launch::application::resolve_claude::{is_usable, resolve_claude};
use crate::modules::launch::domain::claude_candidates::candidate_paths;
use crate::modules::launch::domain::claude_version::parse_version_output;
use crate::modules::launch::error::LaunchError;
use crate::modules::launch::ports::shell_probe::{ShellMode, ShellProbe};
use crate::modules::settings::SettingsRepository;

const PROBE_TIMEOUT: Duration = Duration::from_secs(3);
const VERSION_TIMEOUT: Duration = Duration::from_secs(5);
const LIST_SCRIPT: &str = "which -a claude";

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
    /// More than one usable install, on different versions.
    pub version_mismatch: bool,
}

/// Everything `doctor` reports about `claude`: every install on the PATH,
/// its version and which one a launch would pick.
pub fn inspect_claude(
    settings: &dyn SettingsRepository,
    probe: &dyn ShellProbe,
) -> ClaudeInspection {
    let (chosen, warning) = match resolve_claude(settings, probe) {
        Ok(resolved) => (Ok(resolved.path), resolved.warning),
        Err(error) => (Err(error), None),
    };
    let mut paths = probe
        .run(ShellMode::LoginInteractive, LIST_SCRIPT, PROBE_TIMEOUT)
        .map(|output| candidate_paths(&output))
        .unwrap_or_default();
    if let Ok(path) = &chosen
        && !paths.contains(path)
    {
        paths.insert(0, path.clone());
    }
    let installations: Vec<ClaudeInstallation> = paths
        .into_iter()
        .map(|path| ClaudeInstallation {
            version: probe
                .version_output(&path, VERSION_TIMEOUT)
                .as_deref()
                .and_then(parse_version_output),
            chosen: chosen.as_ref().is_ok_and(|picked| *picked == path),
            usable: is_usable(probe, &path),
            path,
        })
        .collect();
    let mut versions: Vec<&str> = installations
        .iter()
        .filter(|install| install.usable)
        .filter_map(|install| install.version.as_deref())
        .collect();
    versions.sort_unstable();
    versions.dedup();
    ClaudeInspection {
        version_mismatch: versions.len() > 1,
        chosen,
        warning,
        installations,
    }
}
