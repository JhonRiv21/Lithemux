//! What is still on its way out of the sessions that were closed: programs
//! that have not ended and launches that were being prepared. Each holds
//! the lock of its session until it is gone, so the session cannot start
//! again before.

use std::collections::HashMap;

use lithemux_core::modules::session::SessionRef;

use super::hosted::Hosted;
use crate::terminal_view::GridSize;

#[derive(Default)]
pub(super) struct Leaving {
    /// Closed terminals whose program has not ended yet.
    programs: Vec<Hosted>,
    /// Per session, how many launches were dropped while being prepared
    /// and have not come back yet.
    launches: HashMap<String, usize>,
    /// Sessions asked for again meanwhile, and how.
    reopen: HashMap<String, (SessionRef, GridSize)>,
}

impl Leaving {
    pub fn holds(&self, session_id: &str) -> bool {
        self.launches.contains_key(session_id)
            || self.programs.iter().any(|p| p.session_id == session_id)
    }

    pub fn program(&mut self, hosted: Hosted) {
        self.programs.push(hosted);
    }

    pub fn launch(&mut self, session_id: &str) {
        *self.launches.entry(session_id.to_string()).or_default() += 1;
    }

    /// The program of that run ended. `false` when it was not one of these.
    pub fn program_left(&mut self, session_id: &str, run: u64) -> bool {
        let before = self.programs.len();
        self.programs
            .retain(|p| !(p.session_id == session_id && p.run == run));
        self.programs.len() != before
    }

    pub fn launch_dropped(&mut self, session_id: &str) {
        if let Some(count) = self.launches.get_mut(session_id) {
            *count -= 1;
            if *count == 0 {
                self.launches.remove(session_id);
            }
        }
    }

    pub fn reopen(&mut self, session_id: &str, target: SessionRef, size: GridSize) {
        self.reopen.insert(session_id.to_string(), (target, size));
    }

    pub fn forget_reopen(&mut self, session_id: &str) {
        self.reopen.remove(session_id);
    }

    pub fn take_reopen(&mut self, session_id: &str) -> Option<(SessionRef, GridSize)> {
        self.reopen.remove(session_id)
    }

    /// Every program still leaving; nothing is waited for any more.
    pub fn take_programs(&mut self) -> Vec<Hosted> {
        self.launches.clear();
        self.reopen.clear();
        std::mem::take(&mut self.programs)
    }
}
