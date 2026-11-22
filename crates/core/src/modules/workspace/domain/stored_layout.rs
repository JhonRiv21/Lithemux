//! The layout as the database keeps it. What comes back is checked: a
//! damaged row must not leave the app without a window to show.

use std::collections::HashSet;

use crate::modules::workspace::domain::layout::{Layout, MAX_PANES};

pub fn to_stored(layout: &Layout) -> String {
    // A layout is plain data; serializing it does not fail.
    serde_json::to_string(layout).unwrap_or_default()
}

/// `None` for anything that is not a layout the app could have saved.
pub fn from_stored(stored: &str) -> Option<Layout> {
    let mut layout: Layout = serde_json::from_str(stored).ok()?;
    layout.root.clamp_shares();
    is_consistent(&layout).then_some(layout)
}

fn is_consistent(layout: &Layout) -> bool {
    let shown = layout.root.panes();
    let mut ids = HashSet::new();
    let ids_are_unique = shown
        .iter()
        .chain(&layout.root.split_ids())
        .all(|id| ids.insert(*id));
    // The panes are exactly the ones the tree shows, each once.
    let stored: HashSet<u32> = layout.panes.iter().map(|pane| pane.id).collect();
    let panes_match_the_tree = stored.len() == layout.panes.len()
        && stored == shown.iter().copied().collect::<HashSet<u32>>();
    let mut sessions = HashSet::new();
    let panes_are_sound = layout.panes.iter().all(|pane| {
        pane.active < pane.tabs.len().max(1)
            && pane.tabs.iter().all(|session| sessions.insert(session))
    });
    ids_are_unique
        && panes_match_the_tree
        && panes_are_sound
        && (1..=MAX_PANES).contains(&shown.len())
        && shown.contains(&layout.active_pane)
}
