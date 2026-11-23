//! The order in which the keyboard goes through the sessions: the one of
//! the sidebar, cards that a folded category hides left out. No gpui.

use super::tree::Tree;

/// The sessions as the sidebar shows them, top to bottom.
pub fn visible_order(tree: &Tree) -> Vec<String> {
    let mut order: Vec<String> = tree
        .pinned
        .iter()
        .map(|card| card.session_id.clone())
        .collect();
    for group in tree.groups.iter().filter(|group| !group.collapsed) {
        order.extend(group.header.iter().map(|card| card.session_id.clone()));
        order.extend(group.cards.iter().map(|card| card.session_id.clone()));
    }
    order
}

/// The session `by` places away from `current` (-1 the one above, 1 the
/// one below), going round at the ends. Without a current one, or with one
/// the sidebar does not show, the first going down and the last going up.
pub fn step(order: &[String], current: Option<&str>, by: isize) -> Option<String> {
    let count = order.len() as isize;
    if count == 0 {
        return None;
    }
    let at = current.and_then(|current| order.iter().position(|id| id == current));
    let next = match at {
        Some(at) => (at as isize + by).rem_euclid(count),
        None if by < 0 => count - 1,
        None => 0,
    };
    order.get(next as usize).cloned()
}
