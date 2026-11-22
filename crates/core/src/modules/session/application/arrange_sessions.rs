use crate::modules::session::domain::card_text::title_from;
use crate::modules::session::error::SessionError;
use crate::modules::session::ports::session_repository::SessionRepository;

/// A custom title for the card. An empty one goes back to the session's
/// name. The slug, which is how Claude knows the session, never changes.
pub fn set_title(
    sessions: &dyn SessionRepository,
    session_id: &str,
    typed: &str,
) -> Result<(), SessionError> {
    sessions.update_title(session_id, title_from(typed).as_deref())
}

/// Pins the session after the ones already pinned. Pinning it again keeps
/// its place.
pub fn pin_session(sessions: &dyn SessionRepository, session_id: &str) -> Result<(), SessionError> {
    let pinned = sessions.list_pinned()?;
    if pinned.iter().any(|session| session.id == session_id) {
        return Ok(());
    }
    let last = pinned.iter().filter_map(|session| session.pin_sort).max();
    sessions.update_pin_sort(session_id, Some(last.unwrap_or(0) + 1))
}

pub fn unpin_session(
    sessions: &dyn SessionRepository,
    session_id: &str,
) -> Result<(), SessionError> {
    sessions.update_pin_sort(session_id, None)
}

/// Puts the sessions of a category in the order given. A session of the
/// category that is not named keeps its place after the named ones; an id
/// from elsewhere is ignored.
pub fn reorder_sessions(
    sessions: &dyn SessionRepository,
    category_id: &str,
    ordered_ids: &[String],
) -> Result<(), SessionError> {
    let current = sessions.list_in(category_id)?;
    let named = ordered_ids
        .iter()
        .filter(|id| current.iter().any(|session| &session.id == *id));
    let unnamed = current
        .iter()
        .map(|session| &session.id)
        .filter(|id| !ordered_ids.contains(id));
    for (place, id) in (1_i64..).zip(named.chain(unnamed)) {
        sessions.update_sort(id, place)?;
    }
    Ok(())
}

/// Puts the pinned sessions in the order given. One that is not named keeps
/// its place after the named ones; one that is not pinned stays unpinned.
pub fn reorder_pins(
    sessions: &dyn SessionRepository,
    ordered_ids: &[String],
) -> Result<(), SessionError> {
    let current = sessions.list_pinned()?;
    let named = ordered_ids
        .iter()
        .filter(|id| current.iter().any(|session| &session.id == *id));
    let unnamed = current
        .iter()
        .map(|session| &session.id)
        .filter(|id| !ordered_ids.contains(id));
    for (place, id) in (1_i64..).zip(named.chain(unnamed)) {
        sessions.update_pin_sort(id, Some(place))?;
    }
    Ok(())
}
