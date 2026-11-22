use serde::{Deserialize, Serialize};

/// A pane and its tabs. A tab is a session, named by its id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pane {
    pub id: u32,
    pub tabs: Vec<String>,
    /// The tab on show; 0 when there is none.
    pub active: usize,
}

impl Pane {
    pub(super) fn empty(id: u32) -> Self {
        Pane {
            id,
            tabs: Vec::new(),
            active: 0,
        }
    }

    pub fn active_session(&self) -> Option<&str> {
        self.tabs.get(self.active).map(String::as_str)
    }

    pub(super) fn position(&self, session: &str) -> Option<usize> {
        self.tabs.iter().position(|tab| tab == session)
    }

    pub(super) fn show(&mut self, session: &str) {
        if let Some(index) = self.position(session) {
            self.active = index;
        }
    }

    /// Adds the tab at `index`, or at the end, and shows it.
    pub(super) fn insert(&mut self, session: &str, index: Option<usize>) {
        let index = index.unwrap_or(self.tabs.len()).min(self.tabs.len());
        self.tabs.insert(index, session.to_string());
        self.active = index;
    }

    /// Takes the tab out. The tab on show stays on show; when it is the one
    /// that goes, its neighbour takes over.
    pub(super) fn remove(&mut self, session: &str) -> bool {
        let Some(index) = self.position(session) else {
            return false;
        };
        self.tabs.remove(index);
        if index < self.active {
            self.active -= 1;
        }
        self.active = self.active.min(self.tabs.len().saturating_sub(1));
        true
    }

    /// Shows the tab `step` places away, going round.
    pub(super) fn cycle(&mut self, step: isize) {
        let count = self.tabs.len() as isize;
        if count > 0 {
            self.active = (self.active as isize + step).rem_euclid(count) as usize;
        }
    }
}
