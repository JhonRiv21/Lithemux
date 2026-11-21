use serde::Serialize;

use crate::modules::category::Category;
use crate::modules::session::domain::kind::Kind;
use crate::modules::session::domain::messaging_name::messaging_name;
use crate::modules::session::domain::profile::Profile;
use crate::modules::session::domain::session_ref::SessionRef;
use crate::modules::session::domain::state::State;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Session {
    pub id: String,
    pub category_id: String,
    pub slug: String,
    pub role: String,
    pub kind: Kind,
    pub profile: Profile,
    pub cwd: Option<String>,
    pub claude_session_id: Option<String>,
    pub model: Option<String>,
    pub keep_alive: bool,
    pub state: State,
    pub claude_version: Option<String>,
    pub last_activity_at: Option<i64>,
    pub created_at: i64,
    pub sort: i64,
}

impl Session {
    pub fn messaging_name(&self, category: &Category) -> String {
        messaging_name(&category.slug, &self.slug)
    }

    pub fn effective_cwd<'a>(&'a self, category: &'a Category) -> &'a str {
        self.cwd.as_deref().unwrap_or(&category.root_dir)
    }
}

#[derive(Debug, Clone)]
pub struct NewSession {
    pub target: SessionRef,
    pub role: String,
    pub kind: Kind,
    pub profile: Profile,
    pub cwd: Option<String>,
    pub model: Option<String>,
}
