use crate::modules::session::domain::card_text::preview_from;
use crate::modules::session::error::SessionError;
use crate::modules::session::ports::session_repository::SessionRepository;

/// Something was said in the session. `message` becomes the card's preview
/// when it has text; `asks_for_attention` counts it as unread.
pub fn note_message(
    sessions: &dyn SessionRepository,
    session_id: &str,
    message: Option<&str>,
    asks_for_attention: bool,
) -> Result<(), SessionError> {
    let preview = message.and_then(preview_from);
    if preview.is_none() && !asks_for_attention {
        return Ok(());
    }
    sessions.note_message(session_id, preview.as_deref(), asks_for_attention)
}

/// Claude is waiting for the user, and this is what for. It counts as
/// unread, and it is kept apart from the preview: it goes when the wait
/// ends, and what was said last is still there.
pub fn note_request(
    sessions: &dyn SessionRepository,
    session_id: &str,
    request: Option<&str>,
) -> Result<(), SessionError> {
    sessions.update_notice(session_id, request.and_then(preview_from).as_deref())?;
    sessions.note_message(session_id, None, true)
}

/// The user is looking at the session: nothing in it is unread.
pub fn mark_read(sessions: &dyn SessionRepository, session_id: &str) -> Result<(), SessionError> {
    sessions.clear_unread(session_id)
}
