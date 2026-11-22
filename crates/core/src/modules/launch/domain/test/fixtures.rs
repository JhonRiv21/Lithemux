use crate::modules::category::Category;
use crate::modules::session::{Kind, Profile, Session, State};

pub const SESSION_ID: &str = "0b0e7d0c-6f0a-4f8e-9d55-0c1f6d0a1111";
pub const CLAUDE_SESSION_ID: &str = "7d1c3f52-9a7e-4c1b-8f3e-2b9d5a6c2222";

pub fn category() -> Category {
    Category {
        id: "cat-1".to_string(),
        slug: "selektal".to_string(),
        name: "Selektal".to_string(),
        root_dir: "/work/selektal".to_string(),
        color: None,
        sort: 1,
        created_at: 0,
        collapsed: false,
        notify: true,
    }
}

pub fn claude_session() -> Session {
    Session {
        id: SESSION_ID.to_string(),
        category_id: "cat-1".to_string(),
        slug: "ejecutora".to_string(),
        role: "implements the phases".to_string(),
        kind: Kind::Member,
        profile: Profile::Claude,
        cwd: None,
        claude_session_id: Some(CLAUDE_SESSION_ID.to_string()),
        model: None,
        keep_alive: false,
        state: State::New,
        claude_version: None,
        last_activity_at: None,
        created_at: 0,
        sort: 1,
        title: None,
        pin_sort: None,
        preview: None,
        unread: 0,
        notice: None,
    }
}
