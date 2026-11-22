use crate::modules::launch::error::LaunchError;
use crate::modules::launch::ports::session_locks::SessionLocks;

/// Fails with who has it when the session is open somewhere: in a
/// terminal or in the app. What is about to remove or change a session
/// under a running `claude` asks here first.
pub fn ensure_closed(locks: &dyn SessionLocks, session_id: &str) -> Result<(), LaunchError> {
    match locks.holder(session_id) {
        Some(pid) => Err(LaunchError::AlreadyOpen { pid: Some(pid) }),
        None => Ok(()),
    }
}
