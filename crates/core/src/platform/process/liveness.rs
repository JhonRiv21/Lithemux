use rustix::io::Errno;
use rustix::process::{Pid, test_kill_process};

/// Whether a process with this pid exists, ours or not.
pub fn is_alive(pid: u32) -> bool {
    let Some(pid) = i32::try_from(pid).ok().and_then(Pid::from_raw) else {
        return false;
    };
    matches!(test_kill_process(pid), Ok(()) | Err(Errno::PERM))
}
