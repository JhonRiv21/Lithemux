//! PLAN §2.2: what the sidebar lets the user do, on a real database in a
//! temporary LITHEMUX_HOME.

use lithemux_app::sidebar::actions::SidebarActions;
use lithemux_app::sidebar::error::SidebarError;
use lithemux_core::modules::category::Category;
use lithemux_core::modules::session::{Kind, Profile, Session, SessionError};
use lithemux_core::platform::store::connection::Db;

use crate::hosted_world::HostedWorld;

struct Board {
    world: HostedWorld,
    db: Db,
}

impl Board {
    fn new() -> Self {
        let world = HostedWorld::new();
        let db = world.db();
        Board { world, db }
    }

    fn actions(&self) -> SidebarActions<'_> {
        SidebarActions {
            db: &self.db,
            home: &self.world.home,
        }
    }

    fn loaded(&self) -> (Vec<Category>, Vec<Session>) {
        self.actions().load().unwrap()
    }

    fn demo(&self) -> Category {
        self.loaded().0.remove(0)
    }

    fn slugs(&self) -> Vec<String> {
        self.loaded().1.into_iter().map(|s| s.slug).collect()
    }
}

// PLAN §2.2: create categories. The user types a name and picks a folder.
#[test]
fn a_category_is_created_from_a_name_and_a_folder() {
    let board = Board::new();
    let created = board
        .actions()
        .create_category("Diana — Calidad", &board.world.work())
        .unwrap();
    assert_eq!(created.slug, "diana-calidad");
    assert_eq!(created.name, "Diana — Calidad");
    assert_eq!(created.root_dir, board.world.work().display().to_string());
    // Its board exists, as with `lithemux cat add`.
    assert!(
        board
            .world
            .home
            .path()
            .join("categories/diana-calidad")
            .exists()
    );
    let names: Vec<String> = board.loaded().0.into_iter().map(|c| c.name).collect();
    assert_eq!(names, ["Demo", "Diana — Calidad"]);
}

// PLAN §2.2
#[test]
fn a_second_category_with_the_same_name_gets_a_slug_of_its_own() {
    let board = Board::new();
    let again = board
        .actions()
        .create_category("Demo", &board.world.work())
        .unwrap();
    assert_eq!(again.slug, "demo-2");
}

// PLAN §2.2
#[test]
fn a_category_needs_a_usable_name_and_an_existing_folder() {
    let board = Board::new();
    assert!(matches!(
        board.actions().create_category("¡¡", &board.world.work()),
        Err(SidebarError::UnusableName(_))
    ));
    let missing = board.world.work().join("missing");
    assert!(board.actions().create_category("Other", &missing).is_err());
    assert_eq!(board.loaded().0.len(), 1);
}

// PLAN §2.2: create sessions; the title is the name or a custom title.
#[test]
fn a_session_is_created_from_what_the_user_typed() {
    let board = Board::new();
    let demo = board.demo();
    let (_, plain) = board
        .actions()
        .create_session(&demo.id, "ejecutora", Profile::Claude)
        .unwrap();
    assert_eq!(
        (plain.slug.as_str(), plain.title.as_deref()),
        ("ejecutora", None)
    );
    assert_eq!(plain.kind, Kind::Child);
    assert!(plain.claude_session_id.is_some());

    let (_, titled) = board
        .actions()
        .create_session(&demo.id, "API de pagos", Profile::Claude)
        .unwrap();
    assert_eq!(titled.slug, "api-de-pagos");
    assert_eq!(titled.display_title(), "API de pagos");
}

// PLAN §2.2
#[test]
fn a_session_with_a_name_already_used_is_refused() {
    let board = Board::new();
    let demo = board.demo();
    board
        .actions()
        .create_session(&demo.id, "api", Profile::Claude)
        .unwrap();
    assert!(matches!(
        board
            .actions()
            .create_session(&demo.id, "API", Profile::Claude),
        Err(SidebarError::Session(SessionError::AlreadyExists(_)))
    ));
}

// PLAN §2.3: the «new terminal» button; shell sessions have a card too.
#[test]
fn every_new_terminal_is_a_shell_session_with_a_name_of_its_own() {
    let board = Board::new();
    let demo = board.demo();
    for _ in 0..3 {
        let (_, terminal) = board.actions().create_terminal(&demo.id).unwrap();
        assert_eq!(terminal.profile, Profile::Shell);
    }
    assert_eq!(board.slugs(), ["terminal", "terminal-2", "terminal-3"]);
}

// PLAN §2.2: mark a session as mother, one per category.
#[test]
fn a_category_keeps_a_single_mother() {
    let board = Board::new();
    let first = board.world.add_session("uno", Profile::Claude);
    let second = board.world.add_session("dos", Profile::Claude);
    board.actions().set_mother(&first.id, true).unwrap();
    assert!(matches!(
        board.actions().set_mother(&second.id, true),
        Err(SidebarError::Session(SessionError::MotherExists { .. }))
    ));
    assert_eq!(board.world.stored(&second.id).kind, Kind::Child);

    board.actions().set_mother(&first.id, false).unwrap();
    board.actions().set_mother(&second.id, true).unwrap();
    assert_eq!(board.world.stored(&second.id).kind, Kind::Mother);
}

// PLAN §2.2: keep_alive, pin, rename.
#[test]
fn keep_alive_pin_and_title_are_stored() {
    let board = Board::new();
    let session = board.world.add_session("uno", Profile::Claude);
    board.actions().set_keep_alive(&session.id, true).unwrap();
    board.actions().set_pinned(&session.id, true).unwrap();
    board.actions().rename_session(&session.id, "Uno").unwrap();
    let stored = board.world.stored(&session.id);
    assert!(stored.keep_alive && stored.is_pinned());
    assert_eq!(stored.display_title(), "Uno");

    board.actions().set_pinned(&session.id, false).unwrap();
    assert!(!board.world.stored(&session.id).is_pinned());
}

// PLAN §2.2: reorder sessions, with «move up/down» and by dragging.
#[test]
fn sessions_are_reordered_within_their_category() {
    let board = Board::new();
    let ids: Vec<String> = ["a", "b", "c"]
        .iter()
        .map(|slug| board.world.add_session(slug, Profile::Claude).id)
        .collect();
    board.actions().shift_session(&ids[2], -1).unwrap();
    assert_eq!(board.slugs(), ["a", "c", "b"]);
    board.actions().shift_session(&ids[0], -1).unwrap();
    assert_eq!(board.slugs(), ["a", "c", "b"]);
    board.actions().drop_session_on(&ids[1], &ids[0]).unwrap();
    assert_eq!(board.slugs(), ["b", "a", "c"]);
}

// PLAN §2.2: the pinned ones have an order of their own, apart from the
// one in their category.
#[test]
fn pinned_sessions_are_reordered_among_themselves() {
    let board = Board::new();
    let ids: Vec<String> = ["a", "b", "c"]
        .iter()
        .map(|slug| board.world.add_session(slug, Profile::Claude).id)
        .collect();
    for id in &ids {
        board.actions().set_pinned(id, true).unwrap();
    }
    board.actions().shift_session(&ids[2], -1).unwrap();
    let pinned = |board: &Board| -> Vec<String> {
        let mut sessions = board.loaded().1;
        sessions.sort_by_key(|s| s.pin_sort);
        sessions.into_iter().map(|s| s.slug).collect()
    };
    assert_eq!(pinned(&board), ["a", "c", "b"]);
    assert_eq!(board.slugs(), ["a", "b", "c"]);
}

// PLAN §2.2: rename, reorder, collapse and delete categories.
#[test]
fn categories_are_renamed_reordered_collapsed_and_deleted() {
    let board = Board::new();
    let demo = board.demo();
    let other = board
        .actions()
        .create_category("Other", &board.world.work())
        .unwrap();
    board.world.add_session("uno", Profile::Claude);

    board
        .actions()
        .rename_category(&demo.id, "Demo app")
        .unwrap();
    board.actions().set_collapsed(&demo.id, true).unwrap();
    board.actions().shift_category(&other.id, -1).unwrap();
    let categories = board.loaded().0;
    assert_eq!(categories[0].slug, "other");
    assert_eq!(
        (categories[1].name.as_str(), categories[1].slug.as_str()),
        ("Demo app", "demo")
    );
    assert!(categories[1].collapsed);
    board
        .actions()
        .drop_category_on(&demo.id, &other.id)
        .unwrap();
    assert_eq!(board.loaded().0[0].slug, "demo");

    let gone = board.actions().delete_category(&demo.id).unwrap();
    assert_eq!(gone.len(), 1);
    let (categories, sessions) = board.loaded();
    assert_eq!(categories.len(), 1);
    assert!(sessions.is_empty());
}

// PLAN §2.2: delete sessions. PLAN §2.4: no conversation is ever lost.
#[test]
fn deleting_a_session_leaves_its_conversation_on_disk() {
    let board = Board::new();
    let session = board.world.add_session("uno", Profile::Claude);
    let project = board.world.claude_config().join("projects/-work");
    std::fs::create_dir_all(&project).unwrap();
    let transcript = project.join(format!("{}.jsonl", session.claude_session_id.unwrap()));
    std::fs::write(&transcript, "{}\n").unwrap();

    board.actions().delete_session(&session.id).unwrap();
    assert!(board.slugs().is_empty());
    assert!(transcript.exists());
    assert!(matches!(
        board.actions().delete_session(&session.id),
        Err(SidebarError::Gone)
    ));
}

// Audit of block 3 (6): «move up» moves a card one place among the cards
// the user sees under its category. The mother is the header and a pinned
// session is shown elsewhere: neither is a place to move past.
#[test]
fn moving_a_card_skips_the_siblings_that_are_not_shown_with_it() {
    let board = Board::new();
    let ids: Vec<String> = ["madre", "b", "fijada", "d"]
        .iter()
        .map(|slug| board.world.add_session(slug, Profile::Claude).id)
        .collect();
    board.actions().set_mother(&ids[0], true).unwrap();
    board.actions().set_pinned(&ids[2], true).unwrap();
    let shown = |board: &Board| -> Vec<String> {
        let sessions = board.loaded().1;
        let cards = sessions
            .into_iter()
            .filter(|s| !s.is_pinned() && s.kind != Kind::Mother);
        cards.map(|s| s.slug).collect()
    };
    assert_eq!(shown(&board), ["b", "d"]);

    board.actions().shift_session(&ids[3], -1).unwrap();
    assert_eq!(shown(&board), ["d", "b"]);
    board.actions().shift_session(&ids[3], 1).unwrap();
    assert_eq!(shown(&board), ["b", "d"]);
    board.actions().drop_session_on(&ids[3], &ids[1]).unwrap();
    assert_eq!(shown(&board), ["d", "b"]);

    // Unpinned, the other one is back where it was among them all.
    board.actions().set_pinned(&ids[2], false).unwrap();
    assert_eq!(shown(&board), ["d", "fijada", "b"]);
}
