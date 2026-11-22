//! The workspace: up to four panes in a split tree, each with its tabs.
//! Pure state; the app paints it and the database keeps it.

use serde::{Deserialize, Serialize};

use crate::modules::workspace::domain::pane::Pane;
use crate::modules::workspace::domain::split_tree::{Axis, Node};
use crate::modules::workspace::error::WorkspaceError;

pub const MAX_PANES: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    pub(super) root: Node,
    pub(super) panes: Vec<Pane>,
    pub(super) active_pane: u32,
}

impl Layout {
    pub fn single() -> Self {
        Layout {
            root: Node::Pane(1),
            panes: vec![Pane::empty(1)],
            active_pane: 1,
        }
    }

    pub fn root(&self) -> &Node {
        &self.root
    }

    pub fn pane(&self, id: u32) -> Option<&Pane> {
        self.panes.iter().find(|pane| pane.id == id)
    }

    /// Left to right and top to bottom.
    pub fn panes_in_order(&self) -> Vec<&Pane> {
        let order = self.root.panes();
        order.iter().filter_map(|id| self.pane(*id)).collect()
    }

    pub fn active_pane(&self) -> &Pane {
        // The focused pane always exists; the first one stands in for it
        // rather than failing if it ever did not.
        self.pane(self.active_pane).unwrap_or(&self.panes[0])
    }

    /// The tab on show in each pane.
    pub fn visible_sessions(&self) -> Vec<&str> {
        self.panes_in_order()
            .into_iter()
            .filter_map(Pane::active_session)
            .collect()
    }

    pub fn pane_of(&self, session: &str) -> Option<u32> {
        self.panes
            .iter()
            .find(|pane| pane.position(session).is_some())
            .map(|pane| pane.id)
    }

    fn pane_mut(&mut self, id: u32) -> Option<&mut Pane> {
        self.panes.iter_mut().find(|pane| pane.id == id)
    }

    fn next_id(&self) -> u32 {
        let panes = self.panes.iter().map(|pane| pane.id);
        panes.chain(self.root.split_ids()).max().unwrap_or(0) + 1
    }

    /// Shows the session: in the tab it already has, or in a new one of the
    /// focused pane. Returns the pane it is in.
    pub fn open(&mut self, session: &str) -> u32 {
        let pane = self.pane_of(session).unwrap_or(self.active_pane);
        self.active_pane = pane;
        if let Some(pane) = self.pane_mut(pane) {
            if pane.position(session).is_some() {
                pane.show(session);
            } else {
                pane.insert(session, None);
            }
        }
        pane
    }

    /// Adds an empty pane after `pane` and focuses it.
    pub fn split(&mut self, pane: u32, axis: Axis) -> Result<u32, WorkspaceError> {
        if self.pane(pane).is_none() {
            return Err(WorkspaceError::UnknownPane(pane));
        }
        if self.panes.len() >= MAX_PANES {
            return Err(WorkspaceError::TooManyPanes);
        }
        let split_id = self.next_id();
        let added = split_id + 1;
        self.root.split_pane(pane, axis, split_id, added);
        self.panes.push(Pane::empty(added));
        self.active_pane = added;
        Ok(added)
    }

    /// Moves the divider of a split; the share stays within limits that
    /// leave both sides usable.
    pub fn resize(&mut self, split: u32, share: u16) {
        self.root.resize(split, share);
    }

    pub fn focus_pane(&mut self, pane: u32) {
        if self.pane(pane).is_some() {
            self.active_pane = pane;
        }
    }

    pub fn focus_next_pane(&mut self) {
        self.cycle_focus(1);
    }

    pub fn focus_previous_pane(&mut self) {
        self.cycle_focus(-1);
    }

    fn cycle_focus(&mut self, step: isize) {
        let order = self.root.panes();
        let current = order.iter().position(|id| *id == self.active_pane);
        let next = (current.unwrap_or(0) as isize + step).rem_euclid(order.len() as isize);
        self.active_pane = order[next as usize];
    }

    pub fn select_tab(&mut self, session: &str) {
        if let Some(pane) = self.pane_of(session) {
            self.open(session);
            self.active_pane = pane;
        }
    }

    /// Shows the tab at that place of the focused pane, when it has one.
    pub fn show_tab_at(&mut self, index: usize) {
        let active = self.active_pane;
        if let Some(pane) = self.pane_mut(active)
            && index < pane.tabs.len()
        {
            pane.active = index;
        }
    }

    pub fn next_tab(&mut self) {
        let active = self.active_pane;
        if let Some(pane) = self.pane_mut(active) {
            pane.cycle(1);
        }
    }

    pub fn previous_tab(&mut self) {
        let active = self.active_pane;
        if let Some(pane) = self.pane_mut(active) {
            pane.cycle(-1);
        }
    }

    /// Closes the session's tab. A pane left without tabs goes too, unless
    /// it is the only one.
    pub fn close_tab(&mut self, session: &str) {
        let Some(pane) = self.pane_of(session) else {
            return;
        };
        if let Some(pane) = self.pane_mut(pane) {
            pane.remove(session);
        }
        self.remove_if_empty(pane);
    }

    /// Closes the pane and returns the sessions that had a tab in it. The
    /// only pane is emptied instead.
    pub fn close_pane(&mut self, pane: u32) -> Vec<String> {
        let Some(closing) = self.pane_mut(pane) else {
            return Vec::new();
        };
        let sessions = std::mem::take(&mut closing.tabs);
        closing.active = 0;
        self.remove_if_empty(pane);
        sessions
    }

    /// Moves the session's tab to `to`, at `index` or at the end, and shows
    /// it there. A pane left without tabs goes.
    pub fn move_tab(&mut self, session: &str, to: u32, index: Option<usize>) {
        let Some(from) = self.pane_of(session) else {
            return;
        };
        if self.pane(to).is_none() {
            return;
        }
        if let Some(pane) = self.pane_mut(from) {
            pane.remove(session);
        }
        if let Some(pane) = self.pane_mut(to) {
            pane.insert(session, index);
        }
        self.active_pane = to;
        self.remove_if_empty(from);
    }

    /// Drops the tabs of the sessions `exists` does not know. A pane of a
    /// split that loses its last tab this way goes too.
    pub fn keep_sessions(&mut self, exists: impl Fn(&str) -> bool) {
        let mut emptied = Vec::new();
        for pane in &mut self.panes {
            let gone: Vec<String> = pane
                .tabs
                .iter()
                .filter(|session| !exists(session))
                .cloned()
                .collect();
            for session in &gone {
                pane.remove(session);
            }
            if !gone.is_empty() && pane.tabs.is_empty() {
                emptied.push(pane.id);
            }
        }
        for pane in emptied {
            self.remove_if_empty(pane);
        }
    }

    fn remove_if_empty(&mut self, pane: u32) {
        let is_empty = self.pane(pane).is_some_and(|pane| pane.tabs.is_empty());
        if !is_empty || !self.root.remove_pane(pane) {
            return;
        }
        self.panes.retain(|kept| kept.id != pane);
        if self.active_pane == pane {
            self.active_pane = self.root.panes()[0];
        }
    }
}
