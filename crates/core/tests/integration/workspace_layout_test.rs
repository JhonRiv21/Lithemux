//! PLAN §2.3: the layout and the content of the panes are kept in the
//! database, and a restart restores them.

use lithemux_core::modules::category::SqliteCategoryRepository;
use lithemux_core::modules::session::{
    Kind, SessionRepository, SqliteSessionRepository, remove_session,
};
use lithemux_core::modules::workspace::{
    Axis, Layout, LayoutRepository, Node, SqliteLayoutRepository, restore_layout, save_layout,
};
use lithemux_core::platform::store::connection::Db;

use crate::support::in_memory_store::InMemoryBoards;
use crate::support::sample_data::{add_category, add_session};
use crate::temp_home::TempHome;

fn exists(db: &Db) -> impl Fn(&str) -> bool + '_ {
    |id| {
        SqliteSessionRepository::new(db)
            .find_by_id(id)
            .unwrap()
            .is_some()
    }
}

// PLAN §2.3
#[test]
fn a_home_without_a_saved_layout_starts_with_one_empty_pane() {
    let db = TempHome::new().db();
    let layout = restore_layout(&SqliteLayoutRepository::new(&db), exists(&db)).unwrap();
    assert_eq!(layout, Layout::single());
}

// PLAN §2.3 and §2.5: after a restart the panes, their tabs and the focus
// are where they were.
#[test]
fn the_layout_survives_closing_and_opening_the_database() {
    let temp = TempHome::new();
    let saved = {
        let db = temp.db();
        let (categories, sessions) = (
            SqliteCategoryRepository::new(&db),
            SqliteSessionRepository::new(&db),
        );
        add_category(&categories, &InMemoryBoards::default(), "demo");
        let ids: Vec<String> = ["demo/a", "demo/b", "demo/c"]
            .iter()
            .map(|target| add_session(&categories, &sessions, target, Kind::Member).id)
            .collect();

        let mut layout = Layout::single();
        let first = layout.active_pane().id;
        layout.open(&ids[0]);
        layout.open(&ids[1]);
        let right = layout.split(first, Axis::Horizontal).unwrap();
        layout.open(&ids[2]);
        let Node::Split { id: split, .. } = layout.root().clone() else {
            panic!("the root is not a split");
        };
        layout.resize(split, 320);
        layout.focus_pane(first);
        assert_ne!(first, right);
        save_layout(&SqliteLayoutRepository::new(&db), &layout).unwrap();
        layout
    };

    let db = temp.db();
    let restored = restore_layout(&SqliteLayoutRepository::new(&db), exists(&db)).unwrap();
    assert_eq!(restored, saved);
}

// PLAN §2.3
#[test]
fn saving_again_replaces_what_was_saved() {
    let db = TempHome::new().db();
    let layouts = SqliteLayoutRepository::new(&db);
    let mut layout = Layout::single();
    save_layout(&layouts, &layout).unwrap();
    layout
        .split(layout.active_pane().id, Axis::Vertical)
        .unwrap();
    save_layout(&layouts, &layout).unwrap();
    assert_eq!(restore_layout(&layouts, |_| true).unwrap(), layout);
}

// PLAN §2.2: deleting a session; its tab does not come back.
#[test]
fn a_session_deleted_since_the_layout_was_saved_has_no_tab() {
    let db = TempHome::new().db();
    let (categories, sessions) = (
        SqliteCategoryRepository::new(&db),
        SqliteSessionRepository::new(&db),
    );
    add_category(&categories, &InMemoryBoards::default(), "demo");
    let kept = add_session(&categories, &sessions, "demo/a", Kind::Member);
    let deleted = add_session(&categories, &sessions, "demo/b", Kind::Member);
    let mut layout = Layout::single();
    layout.open(&kept.id);
    layout.open(&deleted.id);
    let layouts = SqliteLayoutRepository::new(&db);
    save_layout(&layouts, &layout).unwrap();

    remove_session(&sessions, &deleted).unwrap();
    let restored = restore_layout(&layouts, exists(&db)).unwrap();
    assert_eq!(restored.active_pane().tabs, std::slice::from_ref(&kept.id));
    assert_eq!(
        restored.active_pane().active_session(),
        Some(kept.id.as_str())
    );
}

// PLAN principle 3: a damaged row does not keep the app from starting.
#[test]
fn a_damaged_layout_is_replaced_by_one_empty_pane() {
    let db = TempHome::new().db();
    let layouts = SqliteLayoutRepository::new(&db);
    layouts.save("{ this is not a layout").unwrap();
    assert_eq!(
        restore_layout(&layouts, |_| true).unwrap(),
        Layout::single()
    );
}
