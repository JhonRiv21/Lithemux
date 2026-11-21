use lithemux_core::modules::category::{
    BoardStore, Category, CategoryRepository, NewCategory, Slug, create_category,
};
use lithemux_core::modules::session::{
    Kind, NewSession, Profile, Session, SessionRef, SessionRepository, create_session,
};

pub const NOW: i64 = 1_800_000_000;

pub fn new_category(slug: &str) -> NewCategory {
    NewCategory {
        slug: Slug::parse(slug).unwrap(),
        name: slug.to_uppercase(),
        root_dir: format!("/work/{slug}"),
        color: None,
    }
}

pub fn new_session(target: &str, kind: Kind) -> NewSession {
    NewSession {
        target: SessionRef::parse(target).unwrap(),
        role: format!("role of {target}"),
        kind,
        profile: Profile::Claude,
        cwd: None,
        model: None,
    }
}

pub fn add_category(
    categories: &dyn CategoryRepository,
    boards: &dyn BoardStore,
    slug: &str,
) -> Category {
    create_category(categories, boards, new_category(slug), NOW).unwrap()
}

pub fn add_session(
    categories: &dyn CategoryRepository,
    sessions: &dyn SessionRepository,
    target: &str,
    kind: Kind,
) -> Session {
    create_session(categories, sessions, new_session(target, kind), NOW)
        .unwrap()
        .1
}
