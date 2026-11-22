//! How the panes share the window: a binary tree whose leaves are panes.

use serde::{Deserialize, Serialize};

pub const MIN_SHARE: u16 = 100;
pub const MAX_SHARE: u16 = 900;
const EVEN_SHARE: u16 = 500;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Axis {
    /// Side by side.
    Horizontal,
    /// One above the other.
    Vertical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Node {
    Pane(u32),
    Split {
        id: u32,
        axis: Axis,
        /// What the first side takes, in thousandths.
        share: u16,
        first: Box<Node>,
        second: Box<Node>,
    },
}

impl Node {
    /// The panes, left to right and top to bottom.
    pub fn panes(&self) -> Vec<u32> {
        match self {
            Node::Pane(id) => vec![*id],
            Node::Split { first, second, .. } => {
                let mut panes = first.panes();
                panes.extend(second.panes());
                panes
            }
        }
    }

    pub fn split_ids(&self) -> Vec<u32> {
        match self {
            Node::Pane(_) => Vec::new(),
            Node::Split {
                id, first, second, ..
            } => {
                let mut ids = vec![*id];
                ids.extend(first.split_ids());
                ids.extend(second.split_ids());
                ids
            }
        }
    }

    /// Puts `added` next to `pane`, after it. Returns whether `pane` was found.
    pub(super) fn split_pane(&mut self, pane: u32, axis: Axis, split_id: u32, added: u32) -> bool {
        match self {
            Node::Pane(id) if *id == pane => {
                *self = Node::Split {
                    id: split_id,
                    axis,
                    share: EVEN_SHARE,
                    first: Box::new(Node::Pane(pane)),
                    second: Box::new(Node::Pane(added)),
                };
                true
            }
            Node::Pane(_) => false,
            Node::Split { first, second, .. } => {
                first.split_pane(pane, axis, split_id, added)
                    || second.split_pane(pane, axis, split_id, added)
            }
        }
    }

    /// Takes `pane` out; what was next to it gets its room. The root pane
    /// cannot be removed. Returns whether it was.
    pub(super) fn remove_pane(&mut self, pane: u32) -> bool {
        let Node::Split { first, second, .. } = self else {
            return false;
        };
        let kept = match (first.as_ref(), second.as_ref()) {
            (Node::Pane(id), _) if *id == pane => second,
            (_, Node::Pane(id)) if *id == pane => first,
            _ => return first.remove_pane(pane) || second.remove_pane(pane),
        };
        *self = std::mem::replace(kept.as_mut(), Node::Pane(pane));
        true
    }

    pub(super) fn resize(&mut self, split: u32, new_share: u16) {
        if let Node::Split {
            id,
            share,
            first,
            second,
            ..
        } = self
        {
            if *id == split {
                *share = new_share.clamp(MIN_SHARE, MAX_SHARE);
            } else {
                first.resize(split, new_share);
                second.resize(split, new_share);
            }
        }
    }

    pub(super) fn clamp_shares(&mut self) {
        if let Node::Split {
            share,
            first,
            second,
            ..
        } = self
        {
            *share = (*share).clamp(MIN_SHARE, MAX_SHARE);
            first.clamp_shares();
            second.clamp_shares();
        }
    }
}
