use crate::modules::category::Category;
use crate::modules::import::domain::cmux_panel::CmuxPanel;
use crate::modules::session::{Kind, Profile, Session, State};

pub const HOME: &str = "/Users/someone";

/// A conversation id for the panel number `n`.
pub fn conversation(n: u32) -> String {
    format!("00000000-0000-4000-8000-{n:012}")
}

pub fn claude_panel(n: u32, folder: &str) -> CmuxPanel {
    CmuxPanel {
        custom_title: None,
        process_title: Some(format!("task {n}")),
        folder: Some(folder.to_string()),
        pinned: false,
        agent: Some("claude".to_string()),
        claude_session_id: Some(conversation(n)),
    }
}

pub fn category(slug: &str, root_dir: &str) -> Category {
    Category {
        id: format!("id-{slug}"),
        slug: slug.to_string(),
        name: slug.to_uppercase(),
        root_dir: root_dir.to_string(),
        color: None,
        sort: 0,
        created_at: 0,
        collapsed: false,
        notify: true,
    }
}

pub fn session(category: &Category, slug: &str, claude_session_id: Option<String>) -> Session {
    Session {
        id: format!("id-{}-{slug}", category.slug),
        category_id: category.id.clone(),
        slug: slug.to_string(),
        role: String::new(),
        kind: Kind::Member,
        profile: Profile::Claude,
        cwd: None,
        claude_session_id,
        model: None,
        keep_alive: false,
        state: State::Idle,
        claude_version: None,
        last_activity_at: None,
        created_at: 0,
        sort: 0,
        title: None,
        pin_sort: None,
        preview: None,
        notice: None,
        unread: 0,
    }
}
