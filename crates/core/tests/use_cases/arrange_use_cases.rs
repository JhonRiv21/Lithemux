//! What the sidebar lets the user arrange (PLAN §2.2).

use lithemux_core::modules::category::{
    CategoryError, list_categories, rename_category, reorder_categories, set_collapsed,
};
use lithemux_core::modules::session::{
    Kind, SessionRepository, find_session_by_id, list_sessions, mark_read, note_message,
    pin_session, reorder_pins, reorder_sessions, set_title, unpin_session,
};

use crate::support::in_memory_store::{InMemoryBoards, InMemoryStore};
use crate::support::sample_data::{add_category, add_session};

fn world() -> InMemoryStore {
    let (store, boards) = (InMemoryStore::new(), InMemoryBoards::default());
    add_category(&store, &boards, "demo");
    add_category(&store, &boards, "selektal");
    for target in ["demo/a", "demo/b", "demo/c", "selektal/x"] {
        add_session(&store, &store, target, Kind::Member);
    }
    store
}

fn id_of(store: &InMemoryStore, category: &str, slug: &str) -> String {
    let category = list_categories(store)
        .unwrap()
        .into_iter()
        .find(|c| c.slug == category)
        .unwrap();
    store.find(&category.id, slug).unwrap().unwrap().id
}

fn slugs_in(store: &InMemoryStore, category: &str) -> Vec<String> {
    let category = list_categories(store)
        .unwrap()
        .into_iter()
        .find(|c| c.slug == category)
        .unwrap();
    list_sessions(store, &category)
        .unwrap()
        .into_iter()
        .map(|s| s.slug)
        .collect()
}

fn pinned_slugs(store: &InMemoryStore) -> Vec<String> {
    store
        .list_pinned()
        .unwrap()
        .into_iter()
        .map(|s| s.slug)
        .collect()
}

// PLAN §2.2: reorder sessions.
#[test]
fn sessions_take_the_order_given() {
    let store = world();
    let category_id = list_categories(&store).unwrap()[0].id.clone();
    let order = ["c", "a", "b"].map(|slug| id_of(&store, "demo", slug));
    reorder_sessions(&store, &category_id, &order).unwrap();
    assert_eq!(slugs_in(&store, "demo"), ["c", "a", "b"]);
}

// PLAN §2.2
#[test]
fn a_session_left_out_of_the_order_goes_after_and_strangers_are_ignored() {
    let store = world();
    let category_id = list_categories(&store).unwrap()[0].id.clone();
    let order = [
        id_of(&store, "demo", "c"),
        id_of(&store, "selektal", "x"),
        "missing".to_string(),
    ];
    reorder_sessions(&store, &category_id, &order).unwrap();
    assert_eq!(slugs_in(&store, "demo"), ["c", "a", "b"]);
    assert_eq!(slugs_in(&store, "selektal"), ["x"]);
}

// PLAN §2.2: reorder categories.
#[test]
fn categories_take_the_order_given() {
    let store = world();
    let mut ids: Vec<String> = list_categories(&store)
        .unwrap()
        .into_iter()
        .map(|c| c.id)
        .collect();
    ids.reverse();
    reorder_categories(&store, &ids).unwrap();
    let slugs: Vec<String> = list_categories(&store)
        .unwrap()
        .into_iter()
        .map(|c| c.slug)
        .collect();
    assert_eq!(slugs, ["selektal", "demo"]);
}

// PLAN §2.2: rename a category. The slug is the address of its sessions.
#[test]
fn renaming_a_category_changes_its_name_and_never_its_slug() {
    let store = world();
    let category = list_categories(&store).unwrap().remove(0);
    rename_category(&store, &category.id, "  Demo   app ").unwrap();
    let renamed = list_categories(&store).unwrap().remove(0);
    assert_eq!(
        (renamed.name.as_str(), renamed.slug.as_str()),
        ("Demo app", "demo")
    );
    assert!(matches!(
        rename_category(&store, &category.id, "   "),
        Err(CategoryError::EmptyName)
    ));
}

// PLAN §2.2: collapsible categories.
#[test]
fn a_category_remembers_that_it_is_collapsed() {
    let store = world();
    let category = list_categories(&store).unwrap().remove(0);
    assert!(!category.collapsed);
    set_collapsed(&store, &category.id, true).unwrap();
    assert!(list_categories(&store).unwrap()[0].collapsed);
}

// PLAN §2.2: pinned sessions.
#[test]
fn pins_keep_the_order_they_were_made_in() {
    let store = world();
    pin_session(&store, &id_of(&store, "selektal", "x")).unwrap();
    pin_session(&store, &id_of(&store, "demo", "b")).unwrap();
    assert_eq!(pinned_slugs(&store), ["x", "b"]);
}

// PLAN §2.2
#[test]
fn pinning_again_keeps_the_place_and_unpinning_frees_it() {
    let store = world();
    let (x, b) = (id_of(&store, "selektal", "x"), id_of(&store, "demo", "b"));
    pin_session(&store, &x).unwrap();
    pin_session(&store, &b).unwrap();
    pin_session(&store, &x).unwrap();
    assert_eq!(pinned_slugs(&store), ["x", "b"]);
    unpin_session(&store, &x).unwrap();
    assert_eq!(pinned_slugs(&store), ["b"]);
    // A pin does not take the session out of its category's order.
    assert_eq!(slugs_in(&store, "demo"), ["a", "b", "c"]);
}

// PLAN §2.2
#[test]
fn pins_take_the_order_given_and_reordering_pins_nothing_new() {
    let store = world();
    let (x, b, a) = (
        id_of(&store, "selektal", "x"),
        id_of(&store, "demo", "b"),
        id_of(&store, "demo", "a"),
    );
    pin_session(&store, &x).unwrap();
    pin_session(&store, &b).unwrap();
    reorder_pins(&store, &[a, b, x]).unwrap();
    assert_eq!(pinned_slugs(&store), ["b", "x"]);
}

// PLAN §2.2: the title is the session name or a custom title.
#[test]
fn a_custom_title_shows_instead_of_the_name_and_an_empty_one_removes_it() {
    let store = world();
    let a = id_of(&store, "demo", "a");
    let session = |store: &InMemoryStore| find_session_by_id(store, store, &a).unwrap().unwrap().1;
    assert_eq!(session(&store).display_title(), "a");
    set_title(&store, &a, "The API\nserver").unwrap();
    assert_eq!(session(&store).display_title(), "The API server");
    assert_eq!(session(&store).slug, "a");
    set_title(&store, &a, "  ").unwrap();
    assert_eq!(session(&store).display_title(), "a");
}

// PLAN §2.2: unread counter and preview.
#[test]
fn messages_count_as_unread_until_the_session_is_looked_at() {
    let store = world();
    let a = id_of(&store, "demo", "a");
    let session = |store: &InMemoryStore| find_session_by_id(store, store, &a).unwrap().unwrap().1;
    note_message(&store, &a, Some("do this"), false).unwrap();
    note_message(&store, &a, Some("done.\nAnything else?"), true).unwrap();
    note_message(&store, &a, None, true).unwrap();
    note_message(&store, &a, Some("  "), false).unwrap();
    assert_eq!(
        session(&store).preview.as_deref(),
        Some("done. Anything else?")
    );
    assert_eq!(session(&store).unread, 2);
    mark_read(&store, &a).unwrap();
    assert_eq!(session(&store).unread, 0);
    assert_eq!(
        session(&store).preview.as_deref(),
        Some("done. Anything else?")
    );
}
