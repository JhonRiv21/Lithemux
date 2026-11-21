use crate::modules::category::{BoardStore, CategoryRepository};
use crate::modules::context::application::resolve_language::resolve_language;
use crate::modules::context::domain::context_input::ContextInput;
use crate::modules::context::domain::render_context::render_context;
use crate::modules::context::error::ContextError;
use crate::modules::session::{Session, SessionRepository, find_session_by_id, list_sessions};
use crate::modules::settings::SettingsRepository;

pub struct ContextPorts<'a> {
    pub categories: &'a dyn CategoryRepository,
    pub sessions: &'a dyn SessionRepository,
    pub settings: &'a dyn SettingsRepository,
    pub boards: &'a dyn BoardStore,
}

/// Built from stored state on every call, so it reflects the category as it
/// is right now.
pub fn build_session_context(
    ports: &ContextPorts<'_>,
    session_id: &str,
    now: i64,
) -> Result<String, ContextError> {
    let (category, me) = find_session_by_id(ports.categories, ports.sessions, session_id)?
        .ok_or_else(|| ContextError::SessionNotFound(session_id.to_string()))?;
    let others: Vec<Session> = list_sessions(ports.sessions, &category)?
        .into_iter()
        .filter(|session| session.id != me.id)
        .collect();
    let board = ports.boards.path(&category.slug);
    Ok(render_context(&ContextInput {
        category: &category,
        me: &me,
        others: &others,
        board: &board,
        now,
        lang: resolve_language(ports.settings)?,
    }))
}
