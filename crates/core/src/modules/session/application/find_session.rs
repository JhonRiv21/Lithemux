use crate::modules::category::{Category, CategoryRepository, find_category_by_id, get_category};
use crate::modules::session::domain::session::Session;
use crate::modules::session::domain::session_ref::SessionRef;
use crate::modules::session::error::SessionError;
use crate::modules::session::ports::session_repository::SessionRepository;

pub fn list_sessions(
    sessions: &dyn SessionRepository,
    category: &Category,
) -> Result<Vec<Session>, SessionError> {
    sessions.list_in(&category.id)
}

/// Every session, those of one category together and in its order.
pub fn list_all_sessions(sessions: &dyn SessionRepository) -> Result<Vec<Session>, SessionError> {
    sessions.list_all()
}

pub fn get_session(
    categories: &dyn CategoryRepository,
    sessions: &dyn SessionRepository,
    target: &SessionRef,
) -> Result<(Category, Session), SessionError> {
    let category = get_category(categories, target.category.as_str())?;
    let session = sessions
        .find(&category.id, target.session.as_str())?
        .ok_or_else(|| SessionError::NotFound(target.to_string()))?;
    Ok((category, session))
}

pub fn find_session_by_id(
    categories: &dyn CategoryRepository,
    sessions: &dyn SessionRepository,
    id: &str,
) -> Result<Option<(Category, Session)>, SessionError> {
    let Some(session) = sessions.find_by_id(id)? else {
        return Ok(None);
    };
    let category = find_category_by_id(categories, &session.category_id)?;
    Ok(category.map(|category| (category, session)))
}
