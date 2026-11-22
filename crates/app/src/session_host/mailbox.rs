//! How the background threads tell the host that something happened. The
//! host itself runs on one thread and reads its mail when it is told to.

use std::sync::mpsc;

use lithemux_core::modules::launch::{LaunchError, PreparedLaunch};

use super::event_socket::HookNews;

pub(super) enum Signal {
    /// A hook wrote something about this session to the database.
    Hook(HookNews),
    /// The launch the host asked for is ready, or could not be prepared.
    Prepared {
        session_id: String,
        run: u64,
        outcome: Box<Result<PreparedLaunch, LaunchError>>,
        warnings: Vec<String>,
    },
    /// The program of this run of the session, and all it left, is gone.
    Exited { session_id: String, run: u64 },
}

#[derive(Clone)]
pub(super) struct Mailbox {
    signals: mpsc::Sender<Signal>,
    changed: async_channel::Sender<()>,
}

impl Mailbox {
    /// The second receiver gets one message, never more than one pending,
    /// whenever there is mail.
    pub fn new() -> (Self, mpsc::Receiver<Signal>, async_channel::Receiver<()>) {
        let (signals, inbox) = mpsc::channel();
        let (changed, changes) = async_channel::bounded(1);
        (Mailbox { signals, changed }, inbox, changes)
    }

    pub fn post(&self, signal: Signal) {
        if self.signals.send(signal).is_ok() {
            // Full means the host was already told.
            let _ = self.changed.try_send(());
        }
    }
}
