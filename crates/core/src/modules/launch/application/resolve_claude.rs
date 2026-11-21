use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::modules::launch::domain::claude_candidates::{candidate_paths, is_injected_wrapper};
use crate::modules::launch::error::LaunchError;
use crate::modules::launch::ports::shell_probe::{ShellMode, ShellProbe};
use crate::modules::settings::{SettingKey, SettingsRepository, read_setting};

const PROBE_TIMEOUT: Duration = Duration::from_secs(3);
const LOCATE_SCRIPT: &str = "command -v claude";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedClaude {
    pub path: PathBuf,
    /// Set when the preferred way of finding `claude` did not work.
    pub warning: Option<String>,
}

/// Finds the `claude` to run. Resolved on every launch and never cached, so
/// an auto-updated `claude` is always the one that runs.
///
/// 1. The `claude_path` setting, when it points to an executable.
/// 2. The interactive login shell, which sees the PATH the user works with.
/// 3. The plain login shell, as a fallback.
pub fn resolve_claude(
    settings: &dyn SettingsRepository,
    probe: &dyn ShellProbe,
) -> Result<ResolvedClaude, LaunchError> {
    let mut warning = None;
    if let Some(fixed) = read_setting(settings, SettingKey::ClaudePath)? {
        let path = PathBuf::from(&fixed);
        if probe.is_executable(&path) {
            return Ok(ResolvedClaude {
                path,
                warning: None,
            });
        }
        warning = Some(format!(
            "the claude_path setting points to {fixed}, which is not an executable; looking in the PATH instead"
        ));
    }
    if let Some(path) = locate(probe, ShellMode::LoginInteractive) {
        return Ok(ResolvedClaude { path, warning });
    }
    let path = locate(probe, ShellMode::Login).ok_or_else(|| {
        LaunchError::ClaudeNotFound(
            "`command -v claude` found nothing usable in the user's shell".to_string(),
        )
    })?;
    let fallback = "the interactive login shell did not report a usable `claude`; \
                    fell back to the plain login shell, which may find an older install";
    Ok(ResolvedClaude {
        path,
        warning: Some(match warning {
            Some(first) => format!("{first}; {fallback}"),
            None => fallback.to_string(),
        }),
    })
}

fn locate(probe: &dyn ShellProbe, mode: ShellMode) -> Option<PathBuf> {
    let output = probe.run(mode, LOCATE_SCRIPT, PROBE_TIMEOUT)?;
    candidate_paths(&output)
        .into_iter()
        .rev()
        .find(|path| is_usable(probe, path))
}

pub(super) fn is_usable(probe: &dyn ShellProbe, path: &Path) -> bool {
    !is_injected_wrapper(path, probe.temp_dir().as_deref()) && probe.is_executable(path)
}
