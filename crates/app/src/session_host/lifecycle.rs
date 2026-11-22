//! Putting to sleep what nobody uses (PLAN §2.4). The decisions are
//! `core`'s, pure; here they are asked about the sessions that run and
//! carried out. Whoever calls gives the time and what is on show.

use lithemux_core::modules::session::{
    Pressure, Profile, Running, Session, SqliteSessionRepository, is_due_for_hibernation,
    list_all_sessions, pick_for_memory_valve,
};

use super::host::SessionHost;

impl SessionHost {
    /// The Claude sessions whose program runs here, as the database has
    /// them now.
    fn running_claudes(&self) -> Vec<Session> {
        let sessions = SqliteSessionRepository::new(&self.db);
        let all = list_all_sessions(&sessions).unwrap_or_default();
        let runs_here =
            |session: &Session| session.profile == Profile::Claude && self.is_running(&session.id);
        all.into_iter().filter(runs_here).collect()
    }

    /// Hibernates every session that has been idle and out of sight for
    /// more than `after_hours`. `visible` are the sessions on show. Returns
    /// the ones it hibernated.
    pub fn hibernate_what_is_due(
        &mut self,
        visible: &[String],
        now: i64,
        after_hours: i64,
    ) -> Vec<String> {
        let running = self.running_claudes();
        let due: Vec<String> = running
            .iter()
            .map(|session| Running {
                session,
                is_visible: visible.contains(&session.id),
            })
            .filter(|running| is_due_for_hibernation(running, now, after_hours))
            .map(|running| running.session.id.clone())
            .collect();
        due.into_iter().filter(|id| self.hibernate(id)).collect()
    }

    /// The memory valve: under pressure, hibernates the idle session out of
    /// sight that has been quiet longest, one a minute at most. Returns the
    /// one it took.
    pub fn relieve_memory(
        &mut self,
        visible: &[String],
        now: i64,
        pressure: Pressure,
    ) -> Option<String> {
        let sessions = self.running_claudes();
        let running: Vec<Running<'_>> = sessions
            .iter()
            .map(|session| Running {
                session,
                is_visible: visible.contains(&session.id),
            })
            .collect();
        let last = self.valve_last_used_at;
        let taken = pick_for_memory_valve(&running, now, last, pressure)?.to_string();
        if !self.hibernate(&taken) {
            return None;
        }
        self.valve_last_used_at = Some(now);
        Some(taken)
    }
}
