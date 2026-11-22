//! What happens to a session once its launch is prepared, and when its
//! program ends.

use lithemux_core::modules::category::SqliteCategoryRepository;
use lithemux_core::modules::launch::{LaunchError, PreparedLaunch};
use lithemux_core::modules::session::{
    Profile, SqliteSessionRepository, State, apply_state, find_session_by_id,
};
use lithemux_core::platform::clock::system_clock::now_unix;
use lithemux_core::platform::log::app_log::Level;

use super::error::HostError;
use super::failure::Failure;
use super::host::SessionHost;
use super::opening::start_terminal;
use super::phase::Phase;

impl SessionHost {
    pub(super) fn launch_prepared(
        &mut self,
        session_id: &str,
        run: u64,
        outcome: Result<PreparedLaunch, LaunchError>,
    ) {
        // Closed, or asked for again, while it was being prepared: the
        // launch is dropped here, and its lock with it.
        if self.pending.get(session_id).is_none_or(|p| p.run != run) {
            // The launch, and its lock, end here.
            drop(outcome);
            self.leaving.launch_dropped(session_id);
            self.settle(session_id);
            return;
        }
        let Some(pending) = self.pending.remove(session_id) else {
            return;
        };
        let started = outcome.map_err(HostError::from).and_then(|launch| {
            start_terminal(
                &self.config,
                launch,
                run,
                pending.size,
                self.mailbox.clone(),
            )
        });
        let phase = match started {
            Ok(hosted) => {
                // An earlier run that ended leaves its place to this one.
                self.hosted.retain(|hosted| hosted.session_id != session_id);
                self.hosted.push(hosted);
                Phase::Running
            }
            Err(HostError::Launch(LaunchError::AlreadyOpen { pid })) => {
                self.log_open_elsewhere(session_id, pid);
                Phase::OpenElsewhere { pid }
            }
            Err(error) => {
                self.log_problem("could not start", session_id, &error);
                Phase::Failed(Failure::from(&error))
            }
        };
        self.phases.insert(session_id.to_string(), phase);
        self.refresh(session_id);
    }

    /// The program is gone and the session is already free for anyone to
    /// open again. Its screen stays.
    pub(super) fn program_ended(&mut self, session_id: &str, run: u64) {
        if self.leaving.program_left(session_id, run) {
            self.settle(session_id);
            return;
        }
        // An earlier run of a session that was opened again is old news.
        let Some(profile) = self
            .hosted
            .iter()
            .find(|hosted| hosted.session_id == session_id && hosted.run == run)
            .map(|hosted| hosted.profile)
        else {
            return;
        };
        self.phases.insert(session_id.to_string(), Phase::Ended);
        self.mark_ended(session_id, profile);
        self.refresh(session_id);
    }

    /// Normally the SessionEnd hook already said so; this covers a `claude`
    /// that was killed. A deliberate hibernation stays.
    pub(super) fn mark_ended(&self, session_id: &str, profile: Profile) {
        if profile != Profile::Claude {
            return;
        }
        let sessions = SqliteSessionRepository::new(&self.db);
        let ended = apply_state(
            &sessions,
            session_id,
            State::Ended,
            None,
            Some(State::Hibernated),
        );
        if let Err(error) = ended {
            self.log_problem("could not record the end of", session_id, &error);
        }
    }

    /// The database is the source of truth; a session it does not know is
    /// forgotten here too.
    pub(super) fn refresh(&mut self, session_id: &str) {
        let categories = SqliteCategoryRepository::new(&self.db);
        let sessions = SqliteSessionRepository::new(&self.db);
        match find_session_by_id(&categories, &sessions, session_id) {
            Ok(Some((_, session))) => {
                self.states.insert(session_id.to_string(), session.state);
            }
            Ok(None) => {
                self.states.remove(session_id);
            }
            Err(error) => self.log_problem("could not read the state of", session_id, &error),
        }
    }

    /// Somebody else has the session's lock: a terminal, or a program an
    /// app that died left running.
    fn log_open_elsewhere(&self, session_id: &str, pid: Option<u32>) {
        let holder = pid.map_or_else(String::new, |pid| format!(" (pid {pid})"));
        let line = format!("{session_id} is open elsewhere{holder}: not started here");
        self.log.write(Level::Warn, &line, now_unix());
    }
}
