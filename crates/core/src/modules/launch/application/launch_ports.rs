use crate::modules::category::CategoryRepository;
use crate::modules::launch::ports::conversation_processes::ConversationProcesses;
use crate::modules::launch::ports::hook_binary::HookBinary;
use crate::modules::launch::ports::launch_notices::LaunchNotices;
use crate::modules::launch::ports::session_locks::SessionLocks;
use crate::modules::launch::ports::shell_probe::ShellProbe;
use crate::modules::launch::ports::transcript_locator::TranscriptLocator;
use crate::modules::session::SessionRepository;
use crate::modules::settings::SettingsRepository;

/// What it takes to decide how a session starts. Running it is somebody
/// else's job: the CLI waits on a child in its terminal, the app owns a PTY.
pub struct LaunchPorts<'a> {
    pub categories: &'a dyn CategoryRepository,
    pub sessions: &'a dyn SessionRepository,
    pub settings: &'a dyn SettingsRepository,
    pub shell: &'a dyn ShellProbe,
    pub notices: &'a dyn LaunchNotices,
    pub hook_binary: &'a dyn HookBinary,
    pub transcripts: &'a dyn TranscriptLocator,
    pub locks: &'a dyn SessionLocks,
    /// Who else runs a conversation, lock or not.
    pub conversations: &'a dyn ConversationProcesses,
}
