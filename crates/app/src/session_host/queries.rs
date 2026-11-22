//! What the host can be asked about its sessions.

use std::rc::Rc;

use lithemux_core::modules::category::SqliteCategoryRepository;
use lithemux_core::modules::session::{
    SessionRef, SqliteSessionRepository, State, find_session_by_id,
};
use lithemux_core::platform::clock::system_clock::now_unix;
use lithemux_core::platform::log::app_log::Level;

use super::event_socket::HookNews;
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

    /// A problem that does not stop the host: to the log, and to stderr
    /// for whoever started the app from a terminal. Only which session and
    /// what went wrong; never what was said in it.
    pub(super) fn log_problem(&self, what: &str, session_id: &str, error: &dyn std::fmt::Display) {
        let line = format!("{what} {session_id}: {error}");
        eprintln!("lithemux-app: {line}");
        self.log.write(Level::Error, &line, now_unix());
    }

    /// The address of a session: how it is opened.
    pub(super) fn target_of(&self, session_id: &str) -> Option<SessionRef> {
        let categories = SqliteCategoryRepository::new(&self.db);
        let sessions = SqliteSessionRepository::new(&self.db);
        let (category, session) = find_session_by_id(&categories, &sessions, session_id).ok()??;
        SessionRef::parse(&format!("{}/{}", category.slug, session.slug)).ok()
    }

    /// What the hooks said since the last time this was asked, oldest
    /// first. The state they changed is already read.
    pub fn take_news(&mut self) -> Vec<HookNews> {
        std::mem::take(&mut self.news)
    }

    /// What the launcher had to say while opening sessions.
    pub fn take_warnings(&mut self) -> Vec<String> {
        std::mem::take(&mut self.warnings)
    }
}
