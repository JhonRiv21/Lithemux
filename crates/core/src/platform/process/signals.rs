//! Signal handling for a process that waits on an interactive child.

use std::io;

use rustix::process::{Pid, Signal, kill_process};
use signal_hook::consts::{SIGHUP, SIGINT, SIGQUIT, SIGTERM};
use signal_hook::iterator::{Handle, Signals};

/// While alive, SIGINT and SIGQUIT are swallowed (the child shares the
/// foreground process group and gets them from the terminal) and SIGHUP and
/// SIGTERM are forwarded to the child.
pub struct SignalRelay {
    signals: Signals,
}

pub struct ActiveRelay {
    handle: Handle,
}

impl SignalRelay {
    /// Must be installed before the child is spawned, so no signal is lost.
    pub fn install() -> io::Result<Self> {
        Ok(SignalRelay {
            signals: Signals::new([SIGINT, SIGQUIT, SIGHUP, SIGTERM])?,
        })
    }

    pub fn relay_to(self, child_pid: u32) -> ActiveRelay {
        let handle = self.signals.handle();
        let mut signals = self.signals;
        let target = i32::try_from(child_pid).ok().and_then(Pid::from_raw);
        std::thread::spawn(move || {
            for signal in signals.forever() {
                let forwarded = match signal {
                    SIGHUP => Signal::HUP,
                    SIGTERM => Signal::TERM,
                    _ => continue,
                };
                if let Some(pid) = target {
                    let _ = kill_process(pid, forwarded);
                }
            }
        });
        ActiveRelay { handle }
    }
}

impl Drop for ActiveRelay {
    fn drop(&mut self) {
        self.handle.close();
    }
}
