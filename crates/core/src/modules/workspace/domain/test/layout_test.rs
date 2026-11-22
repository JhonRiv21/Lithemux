use crate::modules::workspace::domain::layout::{Layout, MAX_PANES};
use crate::modules::workspace::domain::split_tree::{Axis, Node};
use crate::modules::workspace::error::WorkspaceError;

fn tabs_of(layout: &Layout, pane: u32) -> Vec<&str> {
    layout
        .pane(pane)
        .unwrap()
        .tabs
        .iter()
        .map(String::as_str)
        .collect()
}

fn pane_ids(layout: &Layout) -> Vec<u32> {
    layout.panes_in_order().iter().map(|pane| pane.id).collect()
}

// PLAN §2.3: a single pane by default.
#[test]
fn a_new_workspace_is_one_empty_pane() {
    let layout = Layout::single();
    assert_eq!(layout.panes_in_order().len(), 1);
    assert!(layout.active_pane().tabs.is_empty());
    assert!(matches!(layout.root(), Node::Pane(_)));
}

// PLAN §2.3: open a session from the sidebar in the active pane.
#[test]
fn a_session_opens_as_the_active_tab_of_the_active_pane() {
    let mut layout = Layout::single();
    layout.open("a");
    layout.open("b");
    let pane = layout.active_pane();
    assert_eq!(pane.tabs, ["a", "b"]);
    assert_eq!(pane.active_session(), Some("b"));
}

// PLAN §2.3: one terminal has one view.
#[test]
fn opening_a_session_that_has_a_tab_goes_to_that_tab() {
    let mut layout = Layout::single();
    let first = layout.active_pane().id;
    layout.open("a");
    layout.open("b");
    let second = layout.split(first, Axis::Horizontal).unwrap();
    layout.open("c");
    assert_eq!(layout.active_pane().id, second);

    layout.open("a");
    assert_eq!(layout.active_pane().id, first);
    assert_eq!(layout.active_pane().active_session(), Some("a"));
    assert_eq!(tabs_of(&layout, first), ["a", "b"]);
    assert_eq!(tabs_of(&layout, second), ["c"]);
}

// PLAN §2.3: splits are made when the user asks; the new pane is focused.
#[test]
fn splitting_adds_an_empty_pane_next_to_the_split_one_and_focuses_it() {
    let mut layout = Layout::single();
    let first = layout.active_pane().id;
    layout.open("a");
    let right = layout.split(first, Axis::Horizontal).unwrap();
    assert_eq!(pane_ids(&layout), [first, right]);
    assert_eq!(layout.active_pane().id, right);
    assert!(layout.active_pane().tabs.is_empty());
    let Node::Split { axis, share, .. } = layout.root() else {
        panic!("the root is not a split");
    };
    assert_eq!((*axis, *share), (Axis::Horizontal, 500));
}

// PLAN §2.3: horizontal or vertical, nested.
#[test]
fn a_pane_of_a_split_can_be_split_the_other_way() {
    let mut layout = Layout::single();
    let first = layout.active_pane().id;
    let right = layout.split(first, Axis::Horizontal).unwrap();
    let below = layout.split(first, Axis::Vertical).unwrap();
    assert_eq!(pane_ids(&layout), [first, below, right]);
}

// PLAN §2.3: up to 4 panes.
#[test]
fn a_fifth_pane_is_refused() {
    let mut layout = Layout::single();
    let first = layout.active_pane().id;
    for _ in 1..MAX_PANES {
        layout.split(first, Axis::Horizontal).unwrap();
    }
    assert_eq!(layout.panes_in_order().len(), MAX_PANES);
    let before = layout.clone();
    assert_eq!(
        layout.split(first, Axis::Vertical),
        Err(WorkspaceError::TooManyPanes)
    );
    assert_eq!(layout, before);
}

// PLAN §2.3
#[test]
fn splitting_a_pane_that_does_not_exist_is_refused() {
    let mut layout = Layout::single();
    assert_eq!(
        layout.split(999, Axis::Horizontal),
        Err(WorkspaceError::UnknownPane(999))
    );
}

// PLAN §2.3: dividers can be dragged.
#[test]
fn a_divider_moves_within_limits_that_keep_both_sides_usable() {
    let mut layout = Layout::single();
    let first = layout.active_pane().id;
    layout.split(first, Axis::Horizontal).unwrap();
    let Node::Split { id, .. } = layout.root() else {
        panic!("the root is not a split");
    };
    let split = *id;
    let share = |layout: &Layout| match layout.root() {
        Node::Split { share, .. } => *share,
        Node::Pane(_) => panic!("the root is not a split"),
    };
    layout.resize(split, 300);
    assert_eq!(share(&layout), 300);
    layout.resize(split, 0);
    assert_eq!(share(&layout), 100);
    layout.resize(split, 5000);
    assert_eq!(share(&layout), 900);
}

// PLAN §2.3: shortcut to close a tab.
#[test]
fn closing_a_tab_shows_its_neighbour() {
    let mut layout = Layout::single();
    for session in ["a", "b", "c"] {
        layout.open(session);
    }
    layout.close_tab("c");
    assert_eq!(layout.active_pane().active_session(), Some("b"));
    layout.open("a");
    layout.close_tab("a");
    assert_eq!(layout.active_pane().tabs, ["b"]);
    assert_eq!(layout.active_pane().active_session(), Some("b"));
}

// PLAN §2.3
#[test]
fn closing_a_tab_that_is_not_shown_keeps_the_one_that_is() {
    let mut layout = Layout::single();
    for session in ["a", "b", "c"] {
        layout.open(session);
    }
    layout.close_tab("a");
    assert_eq!(layout.active_pane().active_session(), Some("c"));
}

// PLAN §2.3: one pane by default; a split lives while it is used.
#[test]
fn closing_the_last_tab_of_a_split_pane_removes_the_pane() {
    let mut layout = Layout::single();
    let first = layout.active_pane().id;
    layout.open("a");
    layout.split(first, Axis::Horizontal).unwrap();
    layout.open("b");
    layout.close_tab("b");
    assert_eq!(pane_ids(&layout), [first]);
    assert_eq!(layout.active_pane().id, first);
    assert!(matches!(layout.root(), Node::Pane(_)));
}

// PLAN §2.3
#[test]
fn the_only_pane_stays_when_its_last_tab_closes() {
    let mut layout = Layout::single();
    layout.open("a");
    layout.close_tab("a");
    assert_eq!(layout.panes_in_order().len(), 1);
    assert!(layout.active_pane().tabs.is_empty());
}

// PLAN §2.3: shortcut to close a pane.
#[test]
fn closing_a_pane_gives_back_its_sessions_and_its_room() {
    let mut layout = Layout::single();
    let first = layout.active_pane().id;
    layout.open("a");
    let right = layout.split(first, Axis::Horizontal).unwrap();
    layout.open("b");
    layout.open("c");
    let below = layout.split(right, Axis::Vertical).unwrap();
    layout.open("d");

    assert_eq!(layout.close_pane(right), ["b", "c"]);
    assert_eq!(pane_ids(&layout), [first, below]);
    assert_eq!(tabs_of(&layout, below), ["d"]);
}

// PLAN §2.3
#[test]
fn closing_the_only_pane_empties_it() {
    let mut layout = Layout::single();
    let only = layout.active_pane().id;
    layout.open("a");
    assert_eq!(layout.close_pane(only), ["a"]);
    assert_eq!(pane_ids(&layout), [only]);
    assert!(layout.active_pane().tabs.is_empty());
}

// PLAN §2.3: shortcuts to change focus.
#[test]
fn focus_goes_round_the_panes_in_reading_order() {
    let mut layout = Layout::single();
    let first = layout.active_pane().id;
    let right = layout.split(first, Axis::Horizontal).unwrap();
    let below = layout.split(first, Axis::Vertical).unwrap();
    layout.focus_pane(first);
    layout.focus_next_pane();
    assert_eq!(layout.active_pane().id, below);
    layout.focus_next_pane();
    assert_eq!(layout.active_pane().id, right);
    layout.focus_next_pane();
    assert_eq!(layout.active_pane().id, first);
    layout.focus_previous_pane();
    assert_eq!(layout.active_pane().id, right);
}

// PLAN §2.3: shortcuts to change tab.
#[test]
fn tabs_go_round_within_the_active_pane() {
    let mut layout = Layout::single();
    for session in ["a", "b", "c"] {
        layout.open(session);
    }
    layout.next_tab();
    assert_eq!(layout.active_pane().active_session(), Some("a"));
    layout.previous_tab();
    assert_eq!(layout.active_pane().active_session(), Some("c"));
    layout.previous_tab();
    assert_eq!(layout.active_pane().active_session(), Some("b"));
}

// PLAN §2.3: drag a tab to another pane.
#[test]
fn a_tab_dragged_to_another_pane_moves_there_and_is_shown() {
    let mut layout = Layout::single();
    let first = layout.active_pane().id;
    layout.open("a");
    layout.open("b");
    let right = layout.split(first, Axis::Horizontal).unwrap();
    layout.open("c");

    layout.move_tab("a", right, None);
    assert_eq!(tabs_of(&layout, first), ["b"]);
    assert_eq!(tabs_of(&layout, right), ["c", "a"]);
    assert_eq!(layout.active_pane().id, right);
    assert_eq!(layout.active_pane().active_session(), Some("a"));
    assert_eq!(layout.pane(first).unwrap().active_session(), Some("b"));
}

// PLAN §2.3
#[test]
fn a_tab_can_be_dropped_at_a_place_among_the_others() {
    let mut layout = Layout::single();
    let first = layout.active_pane().id;
    for session in ["a", "b", "c"] {
        layout.open(session);
    }
    layout.move_tab("c", first, Some(0));
    assert_eq!(tabs_of(&layout, first), ["c", "a", "b"]);
    assert_eq!(layout.active_pane().active_session(), Some("c"));
}

// PLAN §2.3
#[test]
fn dragging_away_the_last_tab_of_a_pane_removes_that_pane() {
    let mut layout = Layout::single();
    let first = layout.active_pane().id;
    layout.open("a");
    let right = layout.split(first, Axis::Horizontal).unwrap();
    layout.open("b");
    layout.move_tab("a", right, None);
    assert_eq!(pane_ids(&layout), [right]);
    assert_eq!(tabs_of(&layout, right), ["b", "a"]);
}

// PLAN §2.3
#[test]
fn a_tab_dropped_on_a_pane_that_does_not_exist_stays_where_it_was() {
    let mut layout = Layout::single();
    layout.open("a");
    let before = layout.clone();
    layout.move_tab("a", 999, None);
    layout.move_tab("missing", before.active_pane().id, None);
    assert_eq!(layout, before);
}

// PLAN §2.5: only the sessions of the visible panes matter at start.
#[test]
fn the_visible_sessions_are_the_shown_tab_of_each_pane() {
    let mut layout = Layout::single();
    let first = layout.active_pane().id;
    layout.open("a");
    layout.open("b");
    layout.split(first, Axis::Horizontal).unwrap();
    layout.open("c");
    assert_eq!(layout.visible_sessions(), ["b", "c"]);
}

// PLAN §2.2: a deleted session leaves the workspace.
#[test]
fn sessions_that_no_longer_exist_lose_their_tabs() {
    let mut layout = Layout::single();
    for session in ["a", "b", "c"] {
        layout.open(session);
    }
    layout.keep_sessions(|session| session != "c");
    assert_eq!(layout.active_pane().tabs, ["a", "b"]);
    assert_eq!(layout.active_pane().active_session(), Some("b"));
}

// Audit of block 3 (5), D-41: a pane of a split left without tabs goes,
// also when its sessions were deleted while the app was closed.
#[test]
fn a_pane_of_a_split_whose_sessions_no_longer_exist_goes_with_them() {
    let mut layout = Layout::single();
    let first = layout.active_pane().id;
    layout.open("a");
    let right = layout.split(first, Axis::Horizontal).unwrap();
    layout.open("gone");
    let below = layout.split(right, Axis::Vertical).unwrap();
    layout.open("b");
    layout.focus_pane(right);

    layout.keep_sessions(|session| session != "gone");

    assert_eq!(pane_ids(&layout), [first, below]);
    assert_ne!(layout.active_pane().id, right);
    assert_eq!(layout.visible_sessions(), ["a", "b"]);
}

// Audit of block 3 (5): a pane that was already empty was left so by the
// user (a fresh split); only the ones emptied by the cleanup go.
#[test]
fn a_pane_the_user_left_empty_stays_when_other_sessions_are_dropped() {
    let mut layout = Layout::single();
    let first = layout.active_pane().id;
    layout.open("a");
    let empty = layout.split(first, Axis::Horizontal).unwrap();
    layout.keep_sessions(|_| true);
    assert_eq!(pane_ids(&layout), [first, empty]);
}

// Audit of block 3 (5)
#[test]
fn the_only_pane_stays_when_all_its_sessions_are_gone() {
    let mut layout = Layout::single();
    layout.open("gone");
    layout.keep_sessions(|_| false);
    assert_eq!(layout.panes_in_order().len(), 1);
    assert!(layout.active_pane().tabs.is_empty());
}

// Block 4, part 0 (2): cmd+1…9 shows the tab at that place of the pane in
// focus; a place past the last tab changes nothing.
#[test]
fn a_tab_is_shown_by_its_place_in_the_pane() {
    let mut layout = Layout::single();
    for session in ["a", "b", "c"] {
        layout.open(session);
    }
    layout.show_tab_at(0);
    assert_eq!(layout.active_pane().active_session(), Some("a"));
    layout.show_tab_at(2);
    assert_eq!(layout.active_pane().active_session(), Some("c"));
    layout.show_tab_at(7);
    assert_eq!(layout.active_pane().active_session(), Some("c"));
}
