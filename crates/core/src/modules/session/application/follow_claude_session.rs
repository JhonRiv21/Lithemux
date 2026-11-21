use uuid::Uuid;

use crate::modules::session::domain::profile::Profile;
use crate::modules::session::error::SessionError;
use crate::modules::session::ports::session_repository::SessionRepository;

/// lithemux follows the conversation the terminal actually has open: `/clear`,
/// a fork or an in-session `/resume` change Claude's session id. Returns the
/// previous id when it changed. Nothing is ever deleted.
pub fn follow_claude_session(
    sessions: &dyn SessionRepository,
    session_id: &str,
    reported_id: &str,
) -> Result<Option<String>, SessionError> {
    if Uuid::parse_str(reported_id).is_err() {
        return Ok(None);
    }
    let Some(session) = sessions.find_by_id(session_id)? else {
        return Ok(None);
    };
    if session.profile != Profile::Claude
        || session.claude_session_id.as_deref() == Some(reported_id)
    {
        return Ok(None);
    }
    sessions.update_claude_session_id(session_id, reported_id)?;
    Ok(Some(session.claude_session_id.unwrap_or_default()))
}
