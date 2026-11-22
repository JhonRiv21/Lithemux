use std::path::PathBuf;
use std::time::Duration;

use crate::modules::launch::application::claude_inspection::{
    ClaudeInspection, ClaudeInstallation,
};
use crate::modules::launch::application::resolve_claude::{
    PROBE_TIMEOUT, SHELL_BUDGET, is_usable, resolve_claude_within,
};
use crate::modules::launch::application::shell_budget::ShellBudget;
use crate::modules::launch::domain::claude_candidates::candidate_paths;
use crate::modules::launch::domain::claude_version::parse_version_output;
use crate::modules::launch::ports::shell_probe::{ShellMode, ShellProbe};
use crate::modules::settings::{SettingKey, SettingsRepository, read_setting};

const VERSION_TIMEOUT: Duration = Duration::from_secs(5);
const LIST_SCRIPT: &str = "which -a claude";

/// Everything `doctor` reports about `claude`: every install on the PATH,
/// its version and which one a launch would pick.
pub fn inspect_claude(
    settings: &dyn SettingsRepository,
    probe: &dyn ShellProbe,
) -> ClaudeInspection {
    let budget = ShellBudget::new(SHELL_BUDGET);
    let (chosen, warning) = match resolve_claude_within(settings, probe, &budget) {
        Ok(resolved) => (Ok(resolved.path), resolved.warning),
        Err(error) => (Err(error), None),
    };
    let mut paths = budget
        .ask(
            probe,
            ShellMode::LoginInteractive,
            LIST_SCRIPT,
            PROBE_TIMEOUT,
        )
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
        remembered: read_setting(settings, SettingKey::LastClaudePath)
            .ok()
            .flatten()
            .map(PathBuf::from),
        version_mismatch: versions.len() > 1,
        chosen,
        warning,
        installations,
    }
}
