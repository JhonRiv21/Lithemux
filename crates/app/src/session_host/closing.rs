//! Closing sessions: one, without waiting for it, or all of them on the way
//! out of the app, staying until they are gone.

use std::time::{Duration, Instant};

use super::host::SessionHost;
use super::hosted::Hosted;
use super::phase::Phase;

/// How long leaving the app waits for the programs to be gone: the grace
/// they get after the hangup, the kill, and the same for what they left.
const SHUTDOWN_PATIENCE: Duration = Duration::from_secs(3);
const SHUTDOWN_POLL: Duration = Duration::from_millis(10);

impl SessionHost {
    /// Stops showing the session and ends its program, without waiting for
    /// it: the session is free again once nothing of it is left.
    pub fn close(&mut self, session_id: &str) {
        self.pending.remove(session_id);
        self.phases.remove(session_id);
        let (closing, kept): (Vec<Hosted>, Vec<Hosted>) = std::mem::take(&mut self.hosted)
            .into_iter()
            .partition(|hosted| hosted.session_id == session_id);
        self.hosted = kept;
        for hosted in closing {
            hosted.terminal.terminate();
            self.mark_ended(session_id, hosted.profile);
        }
        self.refresh(session_id);
    }

    /// Ends every program and stays until they are gone, so that none
    /// outlives the app, and stops listening. This is the one place that
    /// waits. The host is of no use afterwards.
    pub fn shut_down(&mut self) {
        // The app may leave without ever dropping the host, so the socket
        // is closed, and its file removed, here.
        self.socket.take();
        self.pending.clear();
        let closing = std::mem::take(&mut self.hosted);
        for hosted in &closing {
            hosted.terminal.terminate();
        }
        let deadline = Instant::now() + SHUTDOWN_PATIENCE;
        while closing.iter().any(|h| !h.terminal.process_is_gone()) && Instant::now() < deadline {
            std::thread::sleep(SHUTDOWN_POLL);
        }
        for hosted in &closing {
            if self.phases.get(&hosted.session_id) == Some(&Phase::Running) {
                self.mark_ended(&hosted.session_id, hosted.profile);
            }
        }
        self.phases.clear();
    }
}

impl Drop for SessionHost {
    fn drop(&mut self) {
        self.shut_down();
    }
}
