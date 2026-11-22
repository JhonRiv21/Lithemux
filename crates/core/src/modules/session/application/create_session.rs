use uuid::Uuid;

use crate::modules::category::{Category, CategoryRepository, get_category, list_categories};
use crate::modules::session::domain::coordinator_policy::check_can_be_coordinator;
use crate::modules::session::domain::kind::Kind;
use crate::modules::session::domain::messaging_name::name_owner;
use crate::modules::session::domain::profile::Profile;
use crate::modules::session::domain::session::{NewSession, Session};
use crate::modules::session::domain::state::State;
use crate::modules::session::error::SessionError;
use crate::modules::session::ports::session_repository::SessionRepository;

pub fn create_session(
    categories: &dyn CategoryRepository,
    sessions: &dyn SessionRepository,
    new: NewSession,
    now: i64,
) -> Result<(Category, Session), SessionError> {
    let category = get_category(categories, new.target.category.as_str())?;
    if sessions
        .find(&category.id, new.target.session.as_str())?
        .is_some()
    {
        return Err(SessionError::AlreadyExists(new.target.to_string()));
    }
    ensure_messaging_name_is_free(categories, sessions, &new.target.messaging_name())?;
    if new.kind == Kind::Coordinator {
        let current = sessions.find_coordinator(&category.id)?;
        check_can_be_coordinator(current.as_ref(), None).map_err(|coordinator| {
            SessionError::CoordinatorExists {
                category: category.slug.clone(),
                coordinator,
            }
        })?;
    }
    let session = Session {
        id: Uuid::new_v4().to_string(),
        category_id: category.id.clone(),
        slug: new.target.session.to_string(),
        role: new.role,
        kind: new.kind,
        profile: new.profile,
        cwd: new.cwd,
        claude_session_id: (new.profile == Profile::Claude).then(|| Uuid::new_v4().to_string()),
        model: new.model,
        keep_alive: false,
        state: State::New,
        claude_version: None,
        last_activity_at: None,
        created_at: now,
        sort: sessions.next_sort(&category.id)?,
        title: None,
        pin_sort: None,
        preview: None,
        notice: None,
        unread: 0,
    };
    sessions.insert(&session)?;
    Ok((category, session))
}

fn ensure_messaging_name_is_free(
    categories: &dyn CategoryRepository,
    sessions: &dyn SessionRepository,
    name: &str,
) -> Result<(), SessionError> {
    for category in list_categories(categories)? {
        let existing = sessions.list_in(&category.id)?;
        let pairs = existing
            .iter()
            .map(|session| (category.slug.as_str(), session.slug.as_str()));
        if let Some(owner) = name_owner(name, pairs) {
            return Err(SessionError::MessagingNameTaken {
                name: name.to_string(),
                owner,
            });
        }
    }
    Ok(())
}
