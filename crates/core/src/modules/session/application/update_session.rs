use crate::modules::category::Category;
use crate::modules::session::domain::coordinator_policy::check_can_be_coordinator;
use crate::modules::session::domain::kind::Kind;
use crate::modules::session::domain::session::Session;
use crate::modules::session::error::SessionError;
use crate::modules::session::ports::session_repository::SessionRepository;

pub fn set_role(
    sessions: &dyn SessionRepository,
    session: &Session,
    role: &str,
) -> Result<(), SessionError> {
    sessions.update_role(&session.id, role)
}

pub fn set_kind(
    sessions: &dyn SessionRepository,
    category: &Category,
    session: &Session,
    kind: Kind,
) -> Result<(), SessionError> {
    if kind == Kind::Coordinator {
        let current = sessions.find_coordinator(&category.id)?;
        check_can_be_coordinator(current.as_ref(), Some(&session.id)).map_err(|coordinator| {
            SessionError::CoordinatorExists {
                category: category.slug.clone(),
                coordinator,
            }
        })?;
    }
    sessions.update_kind(&session.id, kind)
}

pub fn set_keep_alive(
    sessions: &dyn SessionRepository,
    session: &Session,
    keep_alive: bool,
) -> Result<(), SessionError> {
    sessions.update_keep_alive(&session.id, keep_alive)
}

/// `None` goes back to the user's default model.
pub fn set_model(
    sessions: &dyn SessionRepository,
    session: &Session,
    model: Option<&str>,
) -> Result<(), SessionError> {
    sessions.update_model(&session.id, model)
}
