use crate::modules::session::domain::state::State;
use crate::modules::session::error::SessionError;
use crate::modules::session::ports::session_repository::SessionRepository;

/// Applies a state reported by a running session. `preserve` names a state
/// that must not be overwritten (a deliberate hibernation, for instance).
pub fn apply_state(
    sessions: &dyn SessionRepository,
    session_id: &str,
    state: State,
    last_activity_at: Option<i64>,
    preserve: Option<State>,
) -> Result<(), SessionError> {
    sessions.update_state(session_id, state, last_activity_at, preserve)?;
    forget_the_notice_unless_waiting(sessions, session_id, state)
}

/// What Claude was waiting for stands only while it waits.
fn forget_the_notice_unless_waiting(
    sessions: &dyn SessionRepository,
    session_id: &str,
    state: State,
) -> Result<(), SessionError> {
    if state == State::Waiting {
        return Ok(());
    }
    sessions.update_notice(session_id, None)
}

/// Counts as activity and leaves the state as it is.
pub fn record_activity(
    sessions: &dyn SessionRepository,
    session_id: &str,
    at: i64,
) -> Result<(), SessionError> {
    sessions.touch_activity(session_id, at)
}

/// Moves the session to `to` only while it is in `from`.
pub fn move_state(
    sessions: &dyn SessionRepository,
    session_id: &str,
    from: State,
    to: State,
) -> Result<(), SessionError> {
    sessions.move_state(session_id, from, to)?;
    forget_the_notice_unless_waiting(sessions, session_id, to)
}

pub fn record_claude_version(
    sessions: &dyn SessionRepository,
    session_id: &str,
    version: &str,
) -> Result<(), SessionError> {
    sessions.update_claude_version(session_id, Some(version))
}

/// The version of this run could not be read: it is unknown, not the one of
/// the run before.
pub fn forget_claude_version(
    sessions: &dyn SessionRepository,
    session_id: &str,
) -> Result<(), SessionError> {
    sessions.update_claude_version(session_id, None)
}
