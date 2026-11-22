use std::rc::Rc;

use lithemux_core::modules::session::Profile;

use crate::terminal_view::TerminalSession;

/// One session the host started: its terminal, running or ended.
pub(super) struct Hosted {
    pub session_id: String,
    /// Which start of the session this is; a session can be opened again
    /// after it ends.
    pub run: u64,
    pub profile: Profile,
    pub terminal: Rc<TerminalSession>,
    pub wakes: async_channel::Receiver<()>,
}
