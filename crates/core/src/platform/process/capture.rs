use std::ffi::OsStr;
use std::io::{ErrorKind, Read};
use std::path::Path;
use std::process::{Child, ChildStdout, Stdio};
use std::time::{Duration, Instant};

use rustix::io::ioctl_fionbio;
use rustix::process::{Pid, Signal, kill_process_group};

use crate::platform::process::detached::detach_from_terminal;
use crate::platform::process::session_markers::command_without_session_markers;

const POLL: Duration = Duration::from_millis(5);

/// Runs a short command and returns its stdout. `None` when it fails or does
/// not finish before the deadline.
///
/// The command runs in a session of its own, without a terminal (it is
/// never interactive), and its process group is killed as soon as the
/// command is done or late, so nothing it started keeps running.
///
/// The output is what the pipe holds when the command exits, and nothing
/// that arrives later: a job the command left behind may keep the pipe open
/// and keep writing, and none of that is the command's answer.
pub fn stdout_within<S: AsRef<OsStr>>(
    program: &Path,
    args: &[S],
    timeout: Duration,
) -> Option<String> {
    let mut command = command_without_session_markers(program);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    detach_from_terminal(&mut command);
    let mut child = command.spawn().ok()?;
    let collected = collect_until_exit(&mut child, Instant::now() + timeout);
    kill_group(&child);
    if collected.is_none() {
        let _ = child.wait();
    }
    String::from_utf8(collected?).ok()
}

/// The command's stdout up to its successful exit; `None` when it fails or
/// is still running at the deadline. Reading never blocks, so the pipe is
/// emptied while the command runs (it cannot stall on a full pipe) and one
/// last time when it is gone.
fn collect_until_exit(child: &mut Child, deadline: Instant) -> Option<Vec<u8>> {
    let mut stdout = child.stdout.take()?;
    ioctl_fionbio(&stdout, true).ok()?;
    let mut collected = Vec::new();
    loop {
        drain(&mut stdout, &mut collected);
        match child.try_wait() {
            Ok(Some(status)) if status.success() => {
                drain(&mut stdout, &mut collected);
                return Some(collected);
            }
            Ok(Some(_)) | Err(_) => return None,
            Ok(None) if Instant::now() >= deadline => return None,
            Ok(None) => std::thread::sleep(POLL),
        }
    }
}

/// Moves what is in the pipe right now into `collected`.
fn drain(stdout: &mut ChildStdout, collected: &mut Vec<u8>) {
    let mut chunk = [0_u8; 4096];
    loop {
        match stdout.read(&mut chunk) {
            Ok(0) => return,
            Ok(read) => collected.extend_from_slice(&chunk[..read]),
            Err(error) if error.kind() == ErrorKind::Interrupted => {}
            Err(_) => return,
        }
    }
}

fn kill_group(child: &Child) {
    if let Some(group) = i32::try_from(child.id()).ok().and_then(Pid::from_raw) {
        let _ = kill_process_group(group, Signal::KILL);
    }
}
