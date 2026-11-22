//! What the host can be asked about its sessions.

use std::rc::Rc;

use lithemux_core::modules::session::State;

use super::host::SessionHost;
use super::phase::Phase;
use crate::terminal_view::TerminalSession;

impl SessionHost {
    /// The state the database had for the session the last time the host
    /// heard about it.
    pub fn state(&self, session_id: &str) -> Option<State> {
        self.states.get(session_id).copied()
    }

    /// Where a session the host was asked to open stands; `None` for one it
    /// was never asked for, or that was closed.
    pub fn phase(&self, session_id: &str) -> Option<&Phase> {
        self.phases.get(session_id)
    }

    /// Whether the session was asked for while its last run was leaving,
    /// and will open by itself when that is over.
    pub fn reopens_when_closed(&self, session_id: &str) -> bool {
        self.leaving.is_asked_for(session_id)
    }

    pub fn is_running(&self, session_id: &str) -> bool {
        self.phase(session_id) == Some(&Phase::Running)
    }

    /// The terminal of a session opened here, running or ended, and the
    /// channel that says when it has something new to paint.
    pub fn terminal(
        &self,
        session_id: &str,
    ) -> Option<(Rc<TerminalSession>, async_channel::Receiver<()>)> {
        self.hosted
            .iter()
            .find(|hosted| hosted.session_id == session_id)
            .map(|hosted| (Rc::clone(&hosted.terminal), hosted.wakes.clone()))
    }

    /// The sessions whose program runs, with its pid.
    pub fn running_programs(&self) -> Vec<(String, u32)> {
        self.hosted
            .iter()
            .filter(|hosted| self.is_running(&hosted.session_id))
            .filter_map(|hosted| Some((hosted.session_id.clone(), hosted.terminal.process_id()?)))
            .collect()
    }

    /// What the launcher had to say while opening sessions.
    pub fn take_warnings(&mut self) -> Vec<String> {
        std::mem::take(&mut self.warnings)
    }
}
