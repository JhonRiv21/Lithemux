use crate::modules::launch::error::LaunchError;

/// Held for as long as the session runs; released when dropped.
pub trait HeldSessionLock {
    /// Records the pid of the process running the session, for diagnostics.
    fn record_pid(&mut self, pid: u32);
}

/// One exclusive lock per session, so the same conversation is never resumed
/// twice (two processes resuming one session lose messages).
pub trait SessionLocks {
    /// Fails with `LaunchError::AlreadyOpen` when another process holds it.
    fn acquire(&self, session_id: &str) -> Result<Box<dyn HeldSessionLock>, LaunchError>;
    /// `Some(pid)` while a process holds the session; the pid is 0 when it
    /// was not recorded.
    fn holder(&self, session_id: &str) -> Option<u32>;
}
