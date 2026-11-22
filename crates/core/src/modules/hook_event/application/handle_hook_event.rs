use std::fmt::Display;

use crate::modules::context::build_session_context;
use crate::modules::hook_event::application::hook_ports::HookPorts;
use crate::modules::hook_event::domain::hook_event::HookEvent;
use crate::modules::hook_event::domain::hook_input::HookInput;
use crate::modules::hook_event::domain::session_start_output::session_start_output;
use crate::modules::hook_event::domain::transition::{Transition, transition};
use crate::modules::session::{
    SessionError, SessionRepository, apply_state, follow_claude_session, move_state,
    record_activity, record_claude_version,
};

/// Handles one hook invocation and returns what to print on stdout, if
/// anything. It never fails: every problem goes to the hook log, and a failed
/// write does not keep a starting session from receiving its context.
pub fn handle_hook_event(
    ports: &HookPorts<'_>,
    session_id: &str,
    subcommand: &str,
    stdin: &str,
    now: i64,
) -> Option<String> {
    let Some(event) = HookEvent::parse(subcommand) else {
        ports
            .log
            .append(&format!("unknown hook event \"{subcommand}\""));
        return None;
    };
    let input = HookInput::from_json(stdin);
    let sessions = ports.context.sessions;
    let problem = |what: &str, error: &dyn Display| {
        ports
            .log
            .append(&format!("{} {session_id}: {what}: {error}", event.as_str()));
    };

    if event == HookEvent::SessionStart {
        follow_conversation(ports, session_id, &input, &problem);
    }
    if let Some(change) = transition(event, &input)
        && let Err(error) = apply_transition(sessions, session_id, change, now)
    {
        problem("updating the state", &error);
    }
    let stdout = (event == HookEvent::SessionStart)
        .then(|| session_start_stdout(ports, session_id, &input, now, &problem))
        .flatten();
    ports.notifier.notify(session_id, event);
    stdout
}

fn apply_transition(
    sessions: &dyn SessionRepository,
    session_id: &str,
    change: Transition,
    now: i64,
) -> Result<(), SessionError> {
    match change {
        Transition::Set {
            state,
            touches_activity,
            preserves,
        } => apply_state(
            sessions,
            session_id,
            state,
            touches_activity.then_some(now),
            preserves,
        ),
        Transition::Activity => record_activity(sessions, session_id, now),
        Transition::Move { from, to } => move_state(sessions, session_id, from, to),
    }
}

/// The context Claude receives. A failure here leaves the session without
/// context, never without a start.
fn session_start_stdout(
    ports: &HookPorts<'_>,
    session_id: &str,
    input: &HookInput,
    now: i64,
    problem: &dyn Fn(&str, &dyn Display),
) -> Option<String> {
    if let Some(version) = input.version.as_deref()
        && let Err(error) = record_claude_version(ports.context.sessions, session_id, version)
    {
        problem("recording the claude version", &error);
    }
    match build_session_context(&ports.context, session_id, now) {
        Ok(context) => Some(session_start_output(&context)),
        Err(error) => {
            problem("building the context", &error);
            None
        }
    }
}

fn follow_conversation(
    ports: &HookPorts<'_>,
    session_id: &str,
    input: &HookInput,
    problem: &dyn Fn(&str, &dyn Display),
) {
    let Some(reported) = input.session_id.as_deref() else {
        return;
    };
    match follow_claude_session(ports.context.sessions, session_id, reported) {
        Ok(Some(previous)) => ports.log.append(&format!(
            "session-start {session_id}: claude session id {previous} -> {reported} (source: {})",
            input.source.as_deref().unwrap_or("unknown")
        )),
        Ok(None) => {}
        Err(error) => problem("following the claude session id", &error),
    }
}
