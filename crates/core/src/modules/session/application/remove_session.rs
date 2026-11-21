use crate::modules::session::domain::session::Session;
use crate::modules::session::error::SessionError;
use crate::modules::session::ports::session_repository::SessionRepository;

/// Removes the session from lithemux. The Claude transcript is never touched.
pub fn remove_session(
    sessions: &dyn SessionRepository,
    session: &Session,
) -> Result<(), SessionError> {
    sessions.delete(&session.id)
}
