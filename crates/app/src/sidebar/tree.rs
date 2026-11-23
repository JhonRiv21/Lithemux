//! The sidebar as data: pinned sessions on top, then each category with
//! its coordinator as header and the rest hanging from it.

use std::collections::HashMap;
use std::path::Path;

use lithemux_core::modules::category::Category;
use lithemux_core::modules::session::{Kind, Profile, Session};

use super::cwd_label::cwd_label;
use super::status::{Status, session_status};
use crate::session_host::Phase;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    pub session_id: String,
    /// `<category>/<session>`: how the session is opened.
    pub address: String,
    pub category_id: String,
    pub title: String,
    pub preview: Option<String>,
    pub status: Status,
    pub unread: i64,
    pub cwd: String,
    pub ports: Vec<u16>,
    /// The memory of its program and what it started, while it runs here.
    pub memory: Option<u64>,
    pub is_coordinator: bool,
    pub is_pinned: bool,
    pub keep_alive: bool,
    pub is_shell: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    pub category_id: String,
    pub name: String,
    pub collapsed: bool,
    /// Whether a turn its sessions finish out of view rings («Sound when done»).
    pub notify: bool,
    /// The coordinator, when the category has one that is not pinned.
    pub header: Option<Card>,
    pub cards: Vec<Card>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Tree {
    pub pinned: Vec<Card>,
    pub groups: Vec<Group>,
}

/// What the tree is built from. `categories` and each list of `sessions`
/// come ordered; `pinned` is the order of the pinned ones.
pub struct Board<'a> {
    pub categories: &'a [Category],
    pub sessions: &'a [Session],
    pub phases: &'a dyn Fn(&str) -> Option<Phase>,
    pub ports: &'a HashMap<String, Vec<u16>>,
    /// The memory of each session that runs here, in bytes.
    pub memory: &'a HashMap<String, u64>,
    pub home: Option<&'a Path>,
}

/// The card of a session.
fn card_of(board: &Board<'_>, category: &Category, session: &Session) -> Card {
    Card {
        session_id: session.id.clone(),
        address: format!("{}/{}", category.slug, session.slug),
        category_id: category.id.clone(),
        title: session.display_title().to_string(),
        preview: session.card_line().map(str::to_string),
        status: session_status(board.phases, session),
        unread: session.unread,
        cwd: cwd_label(session.effective_cwd(category), board.home),
        ports: board.ports.get(&session.id).cloned().unwrap_or_default(),
        memory: board.memory.get(&session.id).copied(),
        is_coordinator: session.kind == Kind::Coordinator,
        is_pinned: session.is_pinned(),
        keep_alive: session.keep_alive,
        is_shell: session.profile == Profile::Shell,
    }
}

pub fn build(board: &Board<'_>) -> Tree {
    let mut pinned: Vec<(i64, Card)> = Vec::new();
    let mut groups = Vec::new();
    for category in board.categories {
        let mut group = Group {
            category_id: category.id.clone(),
            name: category.name.clone(),
            collapsed: category.collapsed,
            notify: category.notify,
            header: None,
            cards: Vec::new(),
        };
        let own = board
            .sessions
            .iter()
            .filter(|session| session.category_id == category.id);
        for session in own {
            let card = card_of(board, category, session);
            match session.pin_sort {
                Some(place) => pinned.push((place, card)),
                None if card.is_coordinator => group.header = Some(card),
                None => group.cards.push(card),
            }
        }
        groups.push(group);
    }
    pinned.sort_by_key(|(place, _)| *place);
    Tree {
        pinned: pinned.into_iter().map(|(_, card)| card).collect(),
        groups,
    }
}
