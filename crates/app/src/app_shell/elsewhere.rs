//! Sessions whose conversation runs in a program the app did not start: in
//! cmux, or in a terminal with `lithemux open` (PLAN §3.4). Told from the
//! command lines that the memory meter keeps from the sweep it already
//! makes; no sweep of its own. No gpui.

use std::collections::HashMap;

use lithemux_core::modules::launch::conversation_in;
use lithemux_core::modules::session::Session;
use lithemux_core::platform::process::command_lines::CommandLine;

use crate::session_host::Phase;

/// Which process runs each conversation, by the command lines.
pub fn conversations_held(lines: &[CommandLine]) -> HashMap<String, u32> {
    lines
        .iter()
        .filter_map(|line| Some((conversation_in(&line.args)?.to_string(), line.pid)))
        .collect()
}

/// The sessions, by id, whose conversation a process runs while the app
/// does not run them, with that process.
pub fn open_elsewhere(
    sessions: &[Session],
    held: &HashMap<String, u32>,
    runs_here: &dyn Fn(&str) -> bool,
) -> HashMap<String, u32> {
    sessions
        .iter()
        .filter(|session| !runs_here(&session.id))
        .filter_map(|session| {
            let pid = held.get(session.claude_session_id.as_deref()?)?;
            Some((session.id.clone(), *pid))
        })
        .collect()
}

/// What a card shows: what the host knows of the session, unless the host
/// has no program of it and another process runs its conversation. The
/// host's own «open elsewhere» lasts only while that one runs (its pid is
/// alive, or the sweep still sees the conversation); after it, the card
/// says what the database says.
pub fn shown_phase(
    host: Option<&Phase>,
    elsewhere: Option<u32>,
    holder_runs: bool,
) -> Option<Phase> {
    match host {
        Some(Phase::Opening | Phase::Running | Phase::Closing) => host.cloned(),
        Some(Phase::OpenElsewhere { .. }) if holder_runs || elsewhere.is_some() => host.cloned(),
        Some(Phase::OpenElsewhere { .. }) => None,
        _ => elsewhere
            .map(|pid| Phase::OpenElsewhere { pid: Some(pid) })
            .or_else(|| host.cloned()),
    }
}

/// The sessions whose «open elsewhere» holder (its pid, from `holders`)
/// was seen gone. A pid seen gone stays gone: macOS gives pids again, and
/// the process that gets this one later is not the holder. A session that
/// no longer has a holder, or has another one, starts over.
pub fn ended_holders(
    before: &HashMap<String, u32>,
    holders: &[(String, u32)],
    is_alive: &dyn Fn(u32) -> bool,
) -> HashMap<String, u32> {
    holders
        .iter()
        .filter(|(session, pid)| before.get(session) == Some(pid) || !is_alive(*pid))
        .cloned()
        .collect()
}
