use lithemux_core::modules::hook_event::FileHookLog;
use lithemux_core::modules::launch::{
    ClaudeProjectsTranscriptLocator, CommandLineConversations, LaunchEnvironment, LaunchPorts,
    LoginShellProbe, SiblingHookBinary, TerminalSessionRunner, open_session,
};
use lithemux_core::modules::session::SessionRef;
use lithemux_core::platform::paths::home::Home;
use lithemux_core::platform::process::shell::user_shell;

use crate::output::notices::TerminalNotices;
use crate::wiring::Wiring;

pub fn open(target: &str) -> anyhow::Result<u8> {
    let wiring = Wiring::open()?;
    let environment = LaunchEnvironment {
        shell: user_shell(),
        lithemux_home: Home::env_override(),
    };
    let ports = LaunchPorts {
        categories: &wiring.categories(),
        sessions: &wiring.sessions(),
        settings: &wiring.settings(),
        shell: &LoginShellProbe::new(user_shell()),
        notices: &TerminalNotices {
            log: &FileHookLog::new(&wiring.home),
        },
        hook_binary: &SiblingHookBinary,
        transcripts: &ClaudeProjectsTranscriptLocator::from_env(),
        locks: &wiring.locks(),
        conversations: &CommandLineConversations,
    };
    let code = open_session(
        &ports,
        &TerminalSessionRunner,
        &environment,
        &SessionRef::parse(target)?,
    )?;
    Ok(u8::try_from(code).unwrap_or(1))
}
