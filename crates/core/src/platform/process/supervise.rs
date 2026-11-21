use std::io;
use std::os::unix::process::ExitStatusExt;
use std::path::Path;
use std::process::Command;

use crate::platform::process::signals::SignalRelay;

pub struct ChildSpec<'a> {
    pub program: &'a Path,
    pub args: &'a [String],
    pub env: &'a [(String, String)],
    pub cwd: &'a Path,
}

/// Runs an interactive child on the current terminal and waits for it.
/// `on_started` receives the child's pid. Returns the child's exit code, or
/// 128 + signal when a signal killed it.
pub fn run_to_exit(spec: &ChildSpec<'_>, on_started: &mut dyn FnMut(u32)) -> io::Result<i32> {
    let relay = SignalRelay::install()?;
    let mut child = Command::new(spec.program)
        .args(spec.args)
        .envs(spec.env.iter().map(|(key, value)| (key, value)))
        .current_dir(spec.cwd)
        .spawn()?;
    let _relay = relay.relay_to(child.id());
    on_started(child.id());
    let status = child.wait()?;
    Ok(status
        .code()
        .or_else(|| status.signal().map(|signal| 128 + signal))
        .unwrap_or(1))
}
