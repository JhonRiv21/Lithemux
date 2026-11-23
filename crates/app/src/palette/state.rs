//! What the palette holds while it is open: the sessions it searches, what
//! the search found and which one is selected. No gpui.

use super::ranking::{Entry, Index};

/// More would not fit on the screen; the search narrows them down.
pub const MAX_SHOWN: usize = 50;

pub struct PaletteState {
    index: Index,
    found: Vec<Entry>,
    selected: usize,
}

impl PaletteState {
    /// Opens on every session, the most recent first.
    pub fn new(entries: Vec<Entry>) -> Self {
        let mut state = PaletteState {
            index: Index::new(entries),
            found: Vec::new(),
            selected: 0,
        };
        state.search("");
        state
    }

    /// A new search selects its best result.
    pub fn search(&mut self, query: &str) {
        self.found = self
            .index
            .rank(query)
            .into_iter()
            .take(MAX_SHOWN)
            .cloned()
            .collect();
        self.selected = 0;
    }

    /// ↑ is -1 and ↓ is +1; past either end it goes round.
    pub fn move_by(&mut self, step: isize) {
        let count = self.found.len() as isize;
        if count == 0 {
            return;
        }
        self.selected = (self.selected as isize + step).rem_euclid(count) as usize;
    }

    pub fn select(&mut self, index: usize) {
        if index < self.found.len() {
            self.selected = index;
        }
    }

    pub fn found(&self) -> &[Entry] {
        &self.found
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    /// The session Enter jumps to.
    pub fn chosen(&self) -> Option<&str> {
        self.found
            .get(self.selected)
            .map(|entry| entry.session_id.as_str())
    }
}
