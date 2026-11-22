use crate::platform::process::session_markers::command_without_session_markers;
use crate::platform::process::signals::SignalRelay;
use std::io;
use std::os::unix::process::ExitStatusExt;
use std::path::Path;

pub struct ChildSpec<'a> {
    pub program: &'a Path,
    pub args: &'a [String],
    pub env: &'a [(String, String)],
    pub cwd: &'a Path,
}

/// Runs an interactive child on the current terminal and waits for it. The
/// child does not learn that lithemux itself may run inside another session.
/// `on_started` receives the child's pid. Returns the child's exit code, or
/// 128 + signal when a signal killed it.
pub fn run_to_exit(spec: &ChildSpec<'_>, on_started: &mut dyn FnMut(u32)) -> io::Result<i32> {
    let relay = SignalRelay::install()?;
    let mut command = command_without_session_markers(spec.program);
    command.args(spec.args).current_dir(spec.cwd);
    for (name, value) in spec.env {
        command.env(name, value);
    }
    let mut child = command.spawn()?;
    let _relay = relay.relay_to(child.id());
    on_started(child.id());
    let status = child.wait()?;
    Ok(status
        .code()
        .or_else(|| status.signal().map(|signal| 128 + signal))
        .unwrap_or(1))
}
