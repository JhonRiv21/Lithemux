use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::modules::launch::application::resolved_claude::ResolvedClaude;
use crate::modules::launch::application::shell_budget::ShellBudget;
use crate::modules::launch::domain::claude_candidates::{
    candidate_paths, is_called_claude, is_injected_wrapper,
};
use crate::modules::launch::error::LaunchError;
use crate::modules::launch::ports::shell_probe::{ShellMode, ShellProbe};
use crate::modules::settings::{SettingKey, SettingsRepository, read_setting, write_setting};

pub(super) const PROBE_TIMEOUT: Duration = Duration::from_secs(3);
/// All the waiting for the shell that one launch or one `doctor` may do.
pub(super) const SHELL_BUDGET: Duration = Duration::from_secs(4);
const LOCATE_SCRIPT: &str = "command -v claude";

/// Finds the `claude` to run. Resolved on every launch and never cached, so
/// an auto-updated `claude` is always the one that runs.
///
/// 1. The `claude_path` setting, when it points to an executable.
/// 2. The interactive login shell, which sees the PATH the user works with.
/// 3. The `claude` step 2 found last time, when that shell does not answer:
///    the plain login shell may find an older install.
/// 4. The plain login shell, as a last resort.
///
/// Steps 3 and 4 always come with a warning.
pub fn resolve_claude(
    settings: &dyn SettingsRepository,
    probe: &dyn ShellProbe,
) -> Result<ResolvedClaude, LaunchError> {
    resolve_claude_within(settings, probe, &ShellBudget::new(SHELL_BUDGET))
}

/// The same lookup, charged to a budget the caller goes on using.
pub(super) fn resolve_claude_within(
    settings: &dyn SettingsRepository,
    probe: &dyn ShellProbe,
    budget: &ShellBudget,
) -> Result<ResolvedClaude, LaunchError> {
    let mut warnings = Vec::new();
    if let Some(fixed) = read_setting(settings, SettingKey::ClaudePath)? {
        let path = PathBuf::from(&fixed);
        if probe.is_executable(&path) {
            return Ok(resolved(path, warnings));
        }
        warnings.push(format!(
            "the claude_path setting points to {fixed}, which is not an executable; looking in the PATH instead"
        ));
    }
    if let Some(path) = locate(probe, budget, ShellMode::LoginInteractive) {
        remember(settings, &path);
        return Ok(resolved(path, warnings));
    }
    if let Some(path) = remembered(settings, probe)? {
        warnings.push(format!(
            "the interactive login shell did not report a usable `claude`; \
             using the one it reported last time, {}",
            path.display()
        ));
        return Ok(resolved(path, warnings));
    }
    let path = locate(probe, budget, ShellMode::Login).ok_or_else(|| {
        LaunchError::ClaudeNotFound(
            "`command -v claude` found nothing usable in the user's shell".to_string(),
        )
    })?;
    warnings.push(
        "the interactive login shell did not report a usable `claude`; \
         fell back to the plain login shell, which may find an older install"
            .to_string(),
    );
    Ok(resolved(path, warnings))
}

fn resolved(path: PathBuf, warnings: Vec<String>) -> ResolvedClaude {
    ResolvedClaude {
        path,
        warning: (!warnings.is_empty()).then(|| warnings.join("; ")),
    }
}

/// Written only when it changes, so a launch normally writes nothing. Best
/// effort: the memory is a convenience for another day, and a store that is
/// busy right now must not cost this launch.
fn remember(settings: &dyn SettingsRepository, path: &Path) {
    let found = path.display().to_string();
    let known = read_setting(settings, SettingKey::LastClaudePath)
        .ok()
        .flatten();
    if known.as_deref() != Some(found.as_str()) {
        let _ = write_setting(settings, SettingKey::LastClaudePath, &found);
    }
}

fn remembered(
    settings: &dyn SettingsRepository,
    probe: &dyn ShellProbe,
) -> Result<Option<PathBuf>, LaunchError> {
    Ok(read_setting(settings, SettingKey::LastClaudePath)?
        .map(PathBuf::from)
        .filter(|path| is_usable(probe, path)))
}

fn locate(probe: &dyn ShellProbe, budget: &ShellBudget, mode: ShellMode) -> Option<PathBuf> {
    let output = budget.ask(probe, mode, LOCATE_SCRIPT, PROBE_TIMEOUT)?;
    candidate_paths(&output)
        .into_iter()
        .rev()
        .find(|path| is_usable(probe, path))
}

/// Something a launch found by itself may run: a file called `claude`, not
/// an injected wrapper, executable.
pub(super) fn is_usable(probe: &dyn ShellProbe, path: &Path) -> bool {
    is_called_claude(path)
        && !is_injected_wrapper(path, &probe.temporary_roots())
        && probe.is_executable(path)
}
