use lithemux_core::modules::hook_event::HookLog;
use lithemux_core::modules::launch::LaunchNotices;

/// Shows a warning on the terminal and keeps it in the log.
pub struct TerminalNotices<'a> {
    pub log: &'a dyn HookLog,
}

impl LaunchNotices for TerminalNotices<'_> {
    fn warn(&self, message: &str) {
        eprintln!("lithemux: warning: {message}");
        self.log.append(&format!("open: warning: {message}"));
    }
}
