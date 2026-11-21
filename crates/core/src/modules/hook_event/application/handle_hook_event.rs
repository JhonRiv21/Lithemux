use std::fmt::Display;

use crate::modules::context::build_session_context;
use crate::modules::hook_event::application::hook_ports::HookPorts;
use crate::modules::hook_event::domain::hook_event::HookEvent;
use crate::modules::hook_event::domain::hook_input::HookInput;
use crate::modules::hook_event::domain::session_start_output::session_start_output;
use crate::modules::hook_event::domain::transition::transition;
use crate::modules::session::{apply_state, follow_claude_session, record_claude_version};

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
    if let Some(change) = transition(event, &input) {
        let activity = change.touches_activity.then_some(now);
        if let Err(error) = apply_state(
            sessions,
            session_id,
            change.state,
            activity,
            change.preserves,
        ) {
            problem("updating the state", &error);
        }
    }
    let stdout = if event == HookEvent::SessionStart {
        if let Some(version) = input.version.as_deref()
            && let Err(error) = record_claude_version(sessions, session_id, version)
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
    } else {
        None
    };
    ports.notifier.notify(session_id, event);
    stdout
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
