//! PLAN §3.2 on the host's side: which of the sessions that run here are
//! on an older Claude Code, and resuming the idle ones on the new one.

use lithemux_core::modules::launch::{LiveSession, Update, sessions_to_update};
use lithemux_core::modules::session::{Profile, SessionRepository, SqliteSessionRepository};

use super::host::SessionHost;

impl SessionHost {
    /// The Claude sessions whose program runs here, as stored now.
    fn live_sessions(&self) -> Vec<LiveSession> {
        let sessions = SqliteSessionRepository::new(&self.db);
        self.running_programs()
            .into_iter()
            .filter_map(|(id, _)| sessions.find_by_id(&id).ok().flatten())
            .filter(|session| session.profile == Profile::Claude)
            .map(|session| LiveSession {
                id: session.id,
                state: session.state,
                claude_version: session.claude_version,
                keep_alive: session.keep_alive,
            })
            .collect()
    }

    pub fn claude_update(&self, installed: &str) -> Update {
        sessions_to_update(installed, &self.live_sessions())
    }

    /// Restarts the idle ones on the `claude` a launch would use now, and
    /// returns them; the others are left alone.
    pub fn resume_idle_on(&mut self, installed: &str) -> Vec<String> {
        let idle = self.claude_update(installed).idle;
        for id in &idle {
            self.restart(id);
        }
        idle
    }
}
