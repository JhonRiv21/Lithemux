use std::collections::HashMap;
use std::path::Path;

use lithemux_core::modules::category::Category;
use lithemux_core::modules::session::{Kind, Profile, Session, State};

use crate::session_host::Phase;
use crate::sidebar::status::Status;
use crate::sidebar::tree::{Board, Tree, build};

fn category(slug: &str, sort: i64) -> Category {
    Category {
        id: format!("c-{slug}"),
        slug: slug.to_string(),
        name: slug.to_uppercase(),
        root_dir: format!("/Users/ana/{slug}"),
        color: None,
        sort,
        created_at: 0,
        collapsed: false,
        notify: true,
    }
}

fn session(category: &str, slug: &str, kind: Kind) -> Session {
    Session {
        id: format!("s-{category}-{slug}"),
        category_id: format!("c-{category}"),
        slug: slug.to_string(),
        role: String::new(),
        kind,
        profile: Profile::Claude,
        cwd: None,
        claude_session_id: None,
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
        unread: 0,
        notice: None,
    }
}

fn tree_of(categories: &[Category], sessions: &[Session]) -> Tree {
    build(&Board {
        categories,
        sessions,
        phases: &|_| None,
        ports: &HashMap::new(),
        memory: &HashMap::new(),
        home: Some(Path::new("/Users/ana")),
    })
}

fn titles(cards: &[crate::sidebar::tree::Card]) -> Vec<&str> {
    cards.iter().map(|card| card.title.as_str()).collect()
}

// PLAN §2.2: the coordinator is the header of its category and the rest
// hang from it.
#[test]
fn the_coordinator_heads_its_category() {
    let tree = tree_of(
        &[category("demo", 1)],
        &[
            session("demo", "ejecutora", Kind::Member),
            session("demo", "planificadora", Kind::Coordinator),
            session("demo", "auditora", Kind::Member),
        ],
    );
    let group = &tree.groups[0];
    assert_eq!(group.header.as_ref().unwrap().title, "planificadora");
    assert_eq!(titles(&group.cards), ["ejecutora", "auditora"]);
}

// PLAN §2.2: a category without a coordinator shows its sessions at one level.
#[test]
fn a_category_without_a_coordinator_has_no_header() {
    let tree = tree_of(
        &[category("demo", 1)],
        &[
            session("demo", "a", Kind::Member),
            session("demo", "b", Kind::Member),
        ],
    );
    assert_eq!(tree.groups[0].header, None);
    assert_eq!(titles(&tree.groups[0].cards), ["a", "b"]);
}

// PLAN §2.2: pinned sessions go on top, outside their category.
#[test]
fn pinned_sessions_are_on_top_in_their_own_order_and_not_in_their_category() {
    let mut late = session("demo", "late", Kind::Member);
    late.pin_sort = Some(2);
    let mut early = session("other", "early", Kind::Member);
    early.pin_sort = Some(1);
    let tree = tree_of(
        &[category("demo", 1), category("other", 2)],
        &[late, session("demo", "plain", Kind::Member), early],
    );
    assert_eq!(titles(&tree.pinned), ["early", "late"]);
    assert_eq!(titles(&tree.groups[0].cards), ["plain"]);
    assert!(tree.groups[1].cards.is_empty());
    assert!(tree.pinned.iter().all(|card| card.is_pinned));
}

// PLAN §2.2
#[test]
fn a_pinned_coordinator_leaves_its_category_without_header() {
    let mut coordinator = session("demo", "m", Kind::Coordinator);
    coordinator.pin_sort = Some(1);
    let tree = tree_of(
        &[category("demo", 1)],
        &[coordinator, session("demo", "other", Kind::Member)],
    );
    assert_eq!(tree.groups[0].header, None);
    assert!(tree.pinned[0].is_coordinator);
}

// PLAN §2.2: every category is there, in order, with its folded state.
#[test]
fn categories_keep_their_order_and_an_empty_one_is_still_shown() {
    let mut folded = category("zeta", 1);
    folded.collapsed = true;
    let tree = tree_of(
        &[folded, category("alfa", 2)],
        &[session("alfa", "a", Kind::Member)],
    );
    let names: Vec<&str> = tree.groups.iter().map(|g| g.name.as_str()).collect();
    assert_eq!(names, ["ZETA", "ALFA"]);
    assert!(tree.groups[0].collapsed);
    assert!(tree.groups[0].cards.is_empty());
}

// PLAN §2.2: what a card carries.
#[test]
fn a_card_carries_title_preview_status_unread_cwd_and_ports() {
    let mut api = session("demo", "api", Kind::Member);
    api.title = Some("The API".to_string());
    api.preview = Some("done".to_string());
    api.unread = 3;
    api.cwd = Some("/Users/ana/Work/clients/acme/api".to_string());
    api.keep_alive = true;
    let mut shell = session("demo", "sh", Kind::Member);
    shell.profile = Profile::Shell;
    let ports = HashMap::from([(api.id.clone(), vec![3000, 52844])]);
    // PLAN §3.3: memory per session.
    let memory = HashMap::from([(api.id.clone(), 300 * 1024 * 1024)]);
    let tree = build(&Board {
        categories: &[category("demo", 1)],
        sessions: &[api, shell],
        phases: &|id| (id == "s-demo-api").then_some(Phase::Opening),
        ports: &ports,
        memory: &memory,
        home: Some(Path::new("/Users/ana")),
    });
    let card = &tree.groups[0].cards[0];
    assert_eq!(card.title, "The API");
    assert_eq!(card.address, "demo/api");
    assert_eq!(card.preview.as_deref(), Some("done"));
    assert_eq!(card.status, Status::Opening);
    assert_eq!(card.unread, 3);
    assert_eq!(card.cwd, "~/…/acme/api");
    assert_eq!(card.ports, [3000, 52844]);
    assert_eq!(card.memory, Some(300 * 1024 * 1024));
    assert!(card.keep_alive);
    let shell = &tree.groups[0].cards[1];
    assert!(shell.is_shell);
    assert_eq!(shell.cwd, "~/demo");
    assert_eq!(shell.status, Status::NotRunning);
    assert!(shell.ports.is_empty());
    // A session that does not run here has no memory to show.
    assert_eq!(shell.memory, None);
}
