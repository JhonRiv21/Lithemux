//! What can be done to a card or to a category, as the entries of its
//! menu. No gpui: the view paints them and runs the chosen command.

use lithemux_core::modules::session::Profile;

use super::tree::{Card, Group};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuCommand {
    Open(String),
    Close(String),
    RenameSession(String),
    SetPinned(String, bool),
    SetCoordinator(String, bool),
    SetKeepAlive(String, bool),
    ShiftSession(String, isize),
    DeleteSession(String),
    NewSession(String, Profile),
    RenameCategory(String),
    ShiftCategory(String, isize),
    SetCollapsed(String, bool),
    SetNotify(String, bool),
    DeleteCategory(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuItem {
    /// The key of its text in `locales/*.toml`.
    pub label: &'static str,
    pub command: MenuCommand,
}

fn item(label: &'static str, command: MenuCommand) -> MenuItem {
    MenuItem { label, command }
}

pub fn card_menu(card: &Card) -> Vec<MenuItem> {
    let id = || card.session_id.clone();
    let mut items = vec![item("menu.open", MenuCommand::Open(id()))];
    if card.status.is_live() {
        items.push(item("menu.close", MenuCommand::Close(id())));
    }
    items.push(item("menu.rename", MenuCommand::RenameSession(id())));
    items.push(if card.is_pinned {
        item("menu.unpin", MenuCommand::SetPinned(id(), false))
    } else {
        item("menu.pin", MenuCommand::SetPinned(id(), true))
    });
    // Coordinator and keep-alive are about Claude sessions: a shell has no
    // conversation to coordinate or to hibernate.
    if !card.is_shell {
        items.push(if card.is_coordinator {
            item(
                "menu.remove_coordinator",
                MenuCommand::SetCoordinator(id(), false),
            )
        } else {
            item(
                "menu.make_coordinator",
                MenuCommand::SetCoordinator(id(), true),
            )
        });
        items.push(if card.keep_alive {
            item(
                "menu.keep_alive_off",
                MenuCommand::SetKeepAlive(id(), false),
            )
        } else {
            item("menu.keep_alive_on", MenuCommand::SetKeepAlive(id(), true))
        });
    }
    items.push(item("menu.move_up", MenuCommand::ShiftSession(id(), -1)));
    items.push(item("menu.move_down", MenuCommand::ShiftSession(id(), 1)));
    items.push(item("menu.delete", MenuCommand::DeleteSession(id())));
    items
}

pub fn category_menu(group: &Group) -> Vec<MenuItem> {
    let id = || group.category_id.clone();
    vec![
        item(
            "menu.new_claude_session",
            MenuCommand::NewSession(id(), Profile::Claude),
        ),
        item(
            "menu.new_terminal",
            MenuCommand::NewSession(id(), Profile::Shell),
        ),
        item("menu.rename", MenuCommand::RenameCategory(id())),
        if group.notify {
            item("menu.notify_off", MenuCommand::SetNotify(id(), false))
        } else {
            item("menu.notify_on", MenuCommand::SetNotify(id(), true))
        },
        if group.collapsed {
            item("menu.expand", MenuCommand::SetCollapsed(id(), false))
        } else {
            item("menu.collapse", MenuCommand::SetCollapsed(id(), true))
        },
        item("menu.move_up", MenuCommand::ShiftCategory(id(), -1)),
        item("menu.move_down", MenuCommand::ShiftCategory(id(), 1)),
        item("menu.delete", MenuCommand::DeleteCategory(id())),
    ]
}

/// The entry the keyboard highlights after ↑ (-1) or ↓ (1), going round. The
/// first ↓ highlights the first entry and the first ↑ the last.
pub fn highlight_after(current: Option<usize>, count: usize, by: isize) -> Option<usize> {
    if count == 0 {
        return None;
    }
    let count = count as isize;
    let next = match current {
        Some(at) => (at as isize + by).rem_euclid(count),
        None if by < 0 => count - 1,
        None => 0,
    };
    Some(next as usize)
}
