use std::time::Duration;

use crate::modules::launch::application::resolve_claude::resolve_claude;
use crate::modules::launch::domain::claude_version::parse_version_output;
use crate::modules::launch::ports::shell_probe::ShellProbe;
use crate::modules::settings::SettingsRepository;

const VERSION_TIMEOUT: Duration = Duration::from_secs(5);

/// The version of the `claude` a launch would use now: found as a launch
/// finds it, with the same budget of time for the shell. `None` when there
/// is none, or it does not say.
pub fn installed_claude_version(
    settings: &dyn SettingsRepository,
    probe: &dyn ShellProbe,
) -> Option<String> {
    let resolved = resolve_claude(settings, probe).ok()?;
    let output = probe.version_output(&resolved.path, VERSION_TIMEOUT)?;
    parse_version_output(&output)
}
