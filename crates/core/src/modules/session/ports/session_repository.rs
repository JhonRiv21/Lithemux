use crate::modules::session::domain::kind::Kind;
use crate::modules::session::domain::session::Session;
use crate::modules::session::domain::state::State;
use crate::modules::session::error::SessionError;

pub trait SessionRepository {
    fn insert(&self, session: &Session) -> Result<(), SessionError>;
    fn next_sort(&self, category_id: &str) -> Result<i64, SessionError>;
    /// Ordered by `sort`, then creation time.
    fn list_in(&self, category_id: &str) -> Result<Vec<Session>, SessionError>;
    /// Every session, those of one category together and ordered as
    /// `list_in` orders them.
    fn list_all(&self) -> Result<Vec<Session>, SessionError>;
    fn find(&self, category_id: &str, slug: &str) -> Result<Option<Session>, SessionError>;
    fn find_by_id(&self, id: &str) -> Result<Option<Session>, SessionError>;
    fn find_mother(&self, category_id: &str) -> Result<Option<Session>, SessionError>;

    fn update_role(&self, id: &str, role: &str) -> Result<(), SessionError>;
    fn update_kind(&self, id: &str, kind: Kind) -> Result<(), SessionError>;
    fn update_keep_alive(&self, id: &str, keep_alive: bool) -> Result<(), SessionError>;
    fn update_model(&self, id: &str, model: Option<&str>) -> Result<(), SessionError>;
    fn update_claude_version(&self, id: &str, version: &str) -> Result<(), SessionError>;
    fn update_claude_session_id(&self, id: &str, claude_id: &str) -> Result<(), SessionError>;

    /// Sets the state. `last_activity_at` is left alone when `None`. With
    /// `unless`, a session currently in that state is not touched at all.
    /// Unknown ids are ignored.
    fn update_state(
        &self,
        id: &str,
        state: State,
        last_activity_at: Option<i64>,
        unless: Option<State>,
    ) -> Result<(), SessionError>;

    /// Records activity without touching the state. Unknown ids are ignored.
    fn touch_activity(&self, id: &str, at: i64) -> Result<(), SessionError>;

    /// Moves a session to `to` only while it is in `from`; anything else,
    /// unknown ids included, is left alone.
    fn move_state(&self, id: &str, from: State, to: State) -> Result<(), SessionError>;

    fn update_title(&self, id: &str, title: Option<&str>) -> Result<(), SessionError>;
    fn update_sort(&self, id: &str, sort: i64) -> Result<(), SessionError>;
    /// `None` unpins.
    fn update_pin_sort(&self, id: &str, pin_sort: Option<i64>) -> Result<(), SessionError>;
    /// Every pinned session, of any category, ordered by `pin_sort`.
    fn list_pinned(&self) -> Result<Vec<Session>, SessionError>;

    /// Keeps the line as the preview when there is one, and adds one to the
    /// unread count when `asks_for_attention`. Unknown ids are ignored.
    fn note_message(
        &self,
        id: &str,
        preview: Option<&str>,
        asks_for_attention: bool,
    ) -> Result<(), SessionError>;
    fn clear_unread(&self, id: &str) -> Result<(), SessionError>;

    fn delete(&self, id: &str) -> Result<(), SessionError>;
}
