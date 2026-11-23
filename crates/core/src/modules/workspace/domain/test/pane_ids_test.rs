use crate::modules::workspace::domain::layout::Layout;
use crate::modules::workspace::domain::split_tree::Axis;

// PLAN §3.7: what the app keeps by pane (a browser page) must not pass to
// a new pane that happens to get the id of a closed one.
#[test]
fn a_closed_pane_s_id_is_never_given_to_a_new_pane() {
    let mut layout = Layout::single();
    let closed = layout.split(1, Axis::Horizontal).unwrap();
    layout.close_pane(closed);

    let added = layout.split(1, Axis::Horizontal).unwrap();

    assert_ne!(added, closed);
}

// PLAN §3.7: the same when the pane goes because its last session was
// deleted, which is the way the app does not see coming.
#[test]
fn a_pane_emptied_by_deleting_its_session_does_not_lend_its_id() {
    let mut layout = Layout::single();
    layout.open("b");
    let left = layout.split(1, Axis::Horizontal).unwrap();
    layout.open("a");
    layout.keep_sessions(|id| id != "a");
    assert!(layout.pane(left).is_none());

    let added = layout.split(1, Axis::Vertical).unwrap();

    assert_ne!(added, left);
}

// PLAN §2.3: a layout saved before the ids were remembered still opens,
// and its next pane gets an id no pane of it has.
#[test]
fn a_layout_saved_without_the_last_id_gives_ids_after_its_own() {
    let mut layout = Layout::single();
    let second = layout.split(1, Axis::Horizontal).unwrap();
    let mut saved = serde_json::to_value(&layout).unwrap();
    saved.as_object_mut().unwrap().remove("last_id");
    let mut restored: Layout = serde_json::from_value(saved).unwrap();

    let added = restored.split(second, Axis::Vertical).unwrap();

    assert!(added > second);
}
