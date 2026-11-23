use crate::sidebar::keyboard_order::{step, visible_order};
use crate::sidebar::menu::highlight_after;
use crate::sidebar::status::Status;
use crate::sidebar::tree::{Card, Group, Tree};

fn card(id: &str) -> Card {
    Card {
        session_id: id.to_string(),
        address: format!("c/{id}"),
        category_id: "c".to_string(),
        title: id.to_string(),
        preview: None,
        status: Status::Idle,
        unread: 0,
        cwd: String::new(),
        ports: Vec::new(),
        memory: None,
        is_coordinator: false,
        is_pinned: false,
        keep_alive: false,
        is_shell: false,
    }
}

fn group(id: &str, header: Option<&str>, cards: &[&str], collapsed: bool) -> Group {
    Group {
        category_id: id.to_string(),
        name: id.to_string(),
        collapsed,
        notify: true,
        header: header.map(card),
        cards: cards.iter().map(|id| card(id)).collect(),
    }
}

fn tree() -> Tree {
    Tree {
        pinned: vec![card("pinned")],
        groups: vec![
            group("one", Some("coordinator"), &["a", "b"], false),
            group("folded", Some("hidden-head"), &["hidden"], true),
            group("two", None, &["c"], false),
        ],
    }
}

fn ids(list: &[&str]) -> Vec<String> {
    list.iter().map(|id| id.to_string()).collect()
}

// PLAN §3.5: sesión anterior y siguiente en el orden de la barra lateral
#[test]
fn the_order_is_the_sidebar_s_without_what_a_folded_category_hides() {
    assert_eq!(
        visible_order(&tree()),
        ids(&["pinned", "coordinator", "a", "b", "c"])
    );
}

// PLAN §3.5: next and previous, going round at the ends
#[test]
fn next_and_previous_go_round_at_the_ends() {
    let order = visible_order(&tree());

    assert_eq!(step(&order, Some("a"), 1).as_deref(), Some("b"));
    assert_eq!(step(&order, Some("a"), -1).as_deref(), Some("coordinator"));
    assert_eq!(step(&order, Some("c"), 1).as_deref(), Some("pinned"));
    assert_eq!(step(&order, Some("pinned"), -1).as_deref(), Some("c"));
}

// PLAN §3.5: with nothing in focus, or a session the sidebar hides, it
// starts at an end
#[test]
fn without_a_session_in_view_it_starts_at_an_end() {
    let order = visible_order(&tree());

    assert_eq!(step(&order, None, 1).as_deref(), Some("pinned"));
    assert_eq!(step(&order, Some("hidden"), -1).as_deref(), Some("c"));
    assert_eq!(step(&[], None, 1), None);
}

// PLAN §3.5: the menu is used with the keyboard too
#[test]
fn the_arrows_move_through_the_entries_of_a_menu_and_go_round() {
    assert_eq!(highlight_after(None, 3, 1), Some(0));
    assert_eq!(highlight_after(None, 3, -1), Some(2));
    assert_eq!(highlight_after(Some(2), 3, 1), Some(0));
    assert_eq!(highlight_after(Some(0), 3, -1), Some(2));
    assert_eq!(highlight_after(Some(1), 0, 1), None);
}
