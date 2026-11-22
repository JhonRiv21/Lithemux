use crate::workspace::read_policy::sessions_being_read;

// PLAN §2.2: unread means there was activity while it was not being seen.
#[test]
fn the_tabs_on_show_are_read_while_the_window_is_in_front() {
    assert_eq!(sessions_being_read(&["a", "b"], true), ["a", "b"]);
}

// Audit of block 3 (8): a tab on show in a window the user is not looking
// at has not been seen.
#[test]
fn nothing_is_read_while_the_window_is_in_the_background() {
    assert_eq!(sessions_being_read(&["a", "b"], false), Vec::<&str>::new());
}
