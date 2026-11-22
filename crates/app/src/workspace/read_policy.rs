//! Which sessions the user is looking at, so that what they said counts
//! as read. No gpui.

/// `on_show` is the tab on show in each pane.
/// `on_show` is the tab on show in each pane. With the window in the
/// background the user is looking at none of them.
pub fn sessions_being_read<'a>(on_show: &[&'a str], window_is_active: bool) -> Vec<&'a str> {
    if window_is_active {
        on_show.to_vec()
    } else {
        Vec::new()
    }
}
