use crate::modules::category::Category;
use crate::modules::session::{Kind, Profile, Session, State};

pub const NOW: i64 = 1_800_000_000;
pub const BOARD: &str =
    "/Users/x/Library/Application Support/lithemux/categories/selektal/board.md";

pub fn category() -> Category {
    Category {
        id: "cat-1".to_string(),
        slug: "selektal".to_string(),
        name: "Selektal".to_string(),
        root_dir: "/Users/x/Linktic/RecruitLine".to_string(),
        color: None,
        sort: 1,
        created_at: 0,
        collapsed: false,
        notify: true,
    }
}

pub fn session(slug: &str, role: &str, kind: Kind) -> Session {
    Session {
        id: format!("id-{slug}"),
        category_id: "cat-1".to_string(),
        slug: slug.to_string(),
        role: role.to_string(),
        kind,
        profile: Profile::Claude,
        cwd: None,
        claude_session_id: None,
        model: None,
        keep_alive: false,
        state: State::Idle,
        claude_version: None,
        last_activity_at: Some(NOW - 5 * 60),
        created_at: 0,
        sort: 1,
        title: None,
        pin_sort: None,
        preview: None,
        unread: 0,
        notice: None,
    }
}

pub fn planner() -> Session {
    session(
        "planificadora",
        "planifica, revisa y redacta prompts",
        Kind::Coordinator,
    )
}

pub fn executor() -> Session {
    session(
        "ejecutora",
        "implementa las fases que define la planificadora",
        Kind::Member,
    )
}

pub fn documenter() -> Session {
    let mut session = session("documental", "requerimientos", Kind::Member);
    session.state = State::Hibernated;
    session.last_activity_at = Some(NOW - 3 * 86_400);
    session
}
