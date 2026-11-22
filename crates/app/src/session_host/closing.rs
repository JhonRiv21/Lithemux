//! Closing sessions: one, without waiting for it, or all of them on the way
//! out of the app, staying until they are gone.

use std::time::{Duration, Instant};

use lithemux_core::modules::session::{
    Profile, SessionRepository, SqliteSessionRepository, State, apply_state,
};

use super::host::SessionHost;
use super::hosted::Hosted;
use super::phase::Phase;

/// How long a kill may take to empty a process group, and to free its lock.
const KILL_PATIENCE: Duration = Duration::from_secs(2);
const SHUTDOWN_POLL: Duration = Duration::from_millis(10);

impl SessionHost {
    /// Stops showing the session and ends its program, without waiting for
    /// it: the session is free again once nothing of it is left.
    pub fn close(&mut self, session_id: &str) {
        self.leaving.forget_reopen(session_id);
        if self.pending.remove(session_id).is_some() {
            self.leaving.launch(session_id);
        }
        for hosted in self.take_hosted(session_id) {
            hosted.terminal.terminate();
            self.mark_ended(session_id, hosted.profile);
            self.let_leave(hosted);
        }
        self.settle_phase(session_id);
    }

    /// Puts a running Claude session to sleep: the state is written first,
    /// so that whatever its program reports on the way out finds it
    /// hibernated, and then the program is asked to leave, with time to do
    /// it by itself. Returns whether there was such a session here.
    pub fn hibernate(&mut self, session_id: &str) -> bool {
        let is_a_running_claude = self.is_running(session_id)
            && self
                .hosted
                .iter()
                .any(|hosted| hosted.session_id == session_id && hosted.profile == Profile::Claude);
        if !is_a_running_claude {
            return false;
        }
        let sessions = SqliteSessionRepository::new(&self.db);
        if let Err(error) = apply_state(&sessions, session_id, State::Hibernated, None, None) {
            self.log_problem("could not hibernate", session_id, &error);
            return false;
        }
        for hosted in self.take_hosted(session_id) {
            hosted
                .terminal
                .terminate_within(self.config.hibernate_grace);
            self.let_leave(hosted);
        }
        self.settle_phase(session_id);
        true
    }

    fn take_hosted(&mut self, session_id: &str) -> Vec<Hosted> {
        let (taken, kept): (Vec<Hosted>, Vec<Hosted>) = std::mem::take(&mut self.hosted)
            .into_iter()
            .partition(|hosted| hosted.session_id == session_id);
        self.hosted = kept;
        taken
    }

    /// One that already ended has freed its session; the others hold its
    /// lock until they are gone.
    fn let_leave(&mut self, hosted: Hosted) {
        if !hosted.terminal.process_is_gone() {
            self.leaving.program(hosted);
        }
    }

    fn settle_phase(&mut self, session_id: &str) {
        if self.leaving.holds(session_id) {
            self.phases.insert(session_id.to_string(), Phase::Closing);
        } else {
            self.phases.remove(session_id);
        }
        self.refresh(session_id);
    }

    /// Closes every session here that the database no longer has: removed
    /// from another terminal, its program would go on with nothing to show
    /// it. Returns whether there was any.
    pub fn close_removed(&mut self) -> bool {
        let sessions = SqliteSessionRepository::new(&self.db);
        let removed: Vec<String> = self
            .phases
            .keys()
            .filter(|id| matches!(sessions.find_by_id(id), Ok(None)))
            .cloned()
            .collect();
        for session_id in &removed {
            self.close(session_id);
        }
        !removed.is_empty()
    }

    /// Ends every program and stays until they are gone, so that none
    /// outlives the app, and stops listening. This is the one place that
    /// waits. The host is of no use afterwards.
    ///
    /// Leaving is hibernating (PLAN §2.4): every program gets the grace of
    /// a hibernation to leave by itself, all at the same time. What is
    /// still there when it runs out is killed here: the threads that would
    /// have done it die with the app.
    pub fn shut_down(&mut self) {
        // The app may leave without ever dropping the host, so the socket
        // is closed, and its file removed, here.
        self.socket.take();
        self.pending.clear();
        self.hibernate_all_in_the_database();
        let grace = self.config.hibernate_grace;
        let mut closing = std::mem::take(&mut self.hosted);
        for hosted in &closing {
            hosted.terminal.terminate_within(grace);
        }
        // Those were already told to leave, each with a grace of its own.
        closing.extend(self.leaving.take_programs());
        let all_gone = |closing: &[Hosted]| closing.iter().all(|h| h.terminal.process_is_gone());
        wait_until(Instant::now() + grace, || all_gone(&closing));
        for hosted in &closing {
            hosted.terminal.kill_now();
        }
        wait_until(Instant::now() + KILL_PATIENCE, || all_gone(&closing));
        self.phases.clear();
    }
}

fn wait_until(deadline: Instant, done: impl Fn() -> bool) {
    while !done() && Instant::now() < deadline {
        std::thread::sleep(SHUTDOWN_POLL);
    }
}

impl SessionHost {
    /// Leaving the app is hibernating what runs (PLAN principle 3 and
    /// §2.5). Written before the programs are told to go, so that what they
    /// report on the way out does not undo it.
    fn hibernate_all_in_the_database(&self) {
        let sessions = SqliteSessionRepository::new(&self.db);
        let claudes = self.hosted.iter().filter(|hosted| {
            hosted.profile == Profile::Claude && self.is_running(&hosted.session_id)
        });
        for hosted in claudes {
            let id = &hosted.session_id;
            if let Err(error) = apply_state(&sessions, id, State::Hibernated, None, None) {
                self.log_problem("could not hibernate", id, &error);
            }
        }
    }
}

impl Drop for SessionHost {
    fn drop(&mut self) {
        self.shut_down();
    }
}
