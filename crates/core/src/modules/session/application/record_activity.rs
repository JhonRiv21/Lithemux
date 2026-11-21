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
    sessions.update_state(session_id, state, last_activity_at, preserve)
}

pub fn record_claude_version(
    sessions: &dyn SessionRepository,
    session_id: &str,
    version: &str,
) -> Result<(), SessionError> {
    sessions.update_claude_version(session_id, version)
}
