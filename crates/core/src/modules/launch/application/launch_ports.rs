use crate::modules::category::CategoryRepository;
use crate::modules::launch::ports::hook_binary::HookBinary;
use crate::modules::launch::ports::launch_notices::LaunchNotices;
use crate::modules::launch::ports::session_locks::SessionLocks;
use crate::modules::launch::ports::session_runner::SessionRunner;
use crate::modules::launch::ports::shell_probe::ShellProbe;
use crate::modules::launch::ports::transcript_locator::TranscriptLocator;
use crate::modules::session::SessionRepository;
use crate::modules::settings::SettingsRepository;

pub struct LaunchPorts<'a> {
    pub categories: &'a dyn CategoryRepository,
    pub sessions: &'a dyn SessionRepository,
    pub settings: &'a dyn SettingsRepository,
    pub shell: &'a dyn ShellProbe,
    pub notices: &'a dyn LaunchNotices,
    pub hook_binary: &'a dyn HookBinary,
    pub transcripts: &'a dyn TranscriptLocator,
    pub locks: &'a dyn SessionLocks,
    pub runner: &'a dyn SessionRunner,
}
