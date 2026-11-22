//! Starting a process with no terminal controlling it.

use rustix::process::setsid;
use std::io;
use std::os::unix::process::CommandExt;

use crate::platform::process::session_markers::CleanCommand;

/// Makes `command` start in a session of its own, with no controlling
/// terminal, leading its own process group.
///
/// An interactive shell takes over its terminal at startup. Started in a
/// background process group of the user's terminal it is stopped by the
/// kernel the moment it tries, and never answers. Without a terminal it has
/// nothing to take over. The new group can still be killed as a whole.
///
/// This is the only `unsafe` in the workspace (docs/decisiones.md D-27).
#[allow(unsafe_code)]
pub fn detach_from_terminal(command: &mut CleanCommand) {
    // SAFETY: the closure runs in the child between fork and exec, where only
    // async-signal-safe calls are allowed. It makes one system call, `setsid`,
    // which is on that list; it allocates nothing and takes no lock.
    unsafe {
        command
            .as_command_mut()
            .pre_exec(|| setsid().map(drop).map_err(io::Error::from));
    }
}
