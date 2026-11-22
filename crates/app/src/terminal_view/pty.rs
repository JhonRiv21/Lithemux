//! The child process and its pseudo terminal.

use std::ffi::OsString;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, native_pty_system};
use rustix::io::Errno;
use rustix::process::{Pid, Signal, kill_process_group, test_kill_process_group};

use super::child_env::child_env;
use super::error::TerminalError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellCommand {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub cwd: Option<PathBuf>,
    pub env: Vec<(String, String)>,
    /// Start from the parent's environment; `env` is applied on top.
    pub inherit_env: bool,
}

impl ShellCommand {
    pub fn new(program: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            cwd: None,
            env: Vec::new(),
            inherit_env: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridSize {
    pub columns: u16,
    pub lines: u16,
    pub cell_width: u16,
    pub cell_height: u16,
}

impl GridSize {
    /// The classic 80x24, until the view knows its real size.
    pub const INITIAL: Self = Self {
        columns: 80,
        lines: 24,
        cell_width: 8,
        cell_height: 16,
    };

    fn pty_size(self) -> PtySize {
        PtySize {
            rows: self.lines,
            cols: self.columns,
            pixel_width: self.columns.saturating_mul(self.cell_width),
            pixel_height: self.lines.saturating_mul(self.cell_height),
        }
    }
}

/// How long a program gets to leave after the hangup before it is killed.
const HANGUP_GRACE: Duration = Duration::from_millis(200);
const REAP_POLL: Duration = Duration::from_millis(10);
/// How long a kill may take to empty the group before giving up on it.
const KILL_PATIENCE: Duration = Duration::from_secs(1);
const ROOT: &str = "/";

pub struct PtyProcess {
    master: Box<dyn MasterPty + Send>,
    /// The child leads its own session, so its pid names its process group.
    group: Option<Pid>,
    /// Set once the child was waited for.
    reaped: Arc<AtomicBool>,
    /// Set once nothing of the child's process group is left.
    gone: Arc<AtomicBool>,
    hung_up: AtomicBool,
}

pub struct PtyStreams {
    pub reader: Box<dyn Read + Send>,
    pub writer: Box<dyn Write + Send>,
}

impl PtyProcess {
    pub fn spawn(
        command: &ShellCommand,
        size: GridSize,
        on_exit: Box<dyn FnOnce() + Send>,
    ) -> Result<(Self, PtyStreams), TerminalError> {
        Self::spawn_in(command, size, std::env::vars_os(), on_exit)
    }

    /// `inherited` is the environment of the process that starts the
    /// terminal; the program gets what `child_env` makes of it.
    pub(super) fn spawn_in(
        command: &ShellCommand,
        size: GridSize,
        inherited: impl Iterator<Item = (OsString, OsString)>,
        on_exit: Box<dyn FnOnce() + Send>,
    ) -> Result<(Self, PtyStreams), TerminalError> {
        let pair = native_pty_system()
            .openpty(size.pty_size())
            .map_err(|error| TerminalError::OpenPty(error.to_string()))?;
        let child = pair
            .slave
            .spawn_command(builder(command, inherited))
            .map_err(|error| TerminalError::Spawn {
                program: command.program.clone(),
                message: error.to_string(),
            })?;
        let attach = |error: anyhow::Error| TerminalError::Attach(error.to_string());
        let streams = PtyStreams {
            reader: pair.master.try_clone_reader().map_err(attach)?,
            writer: pair.master.take_writer().map_err(attach)?,
        };
        let group = child
            .process_id()
            .and_then(|pid| i32::try_from(pid).ok())
            .and_then(Pid::from_raw);
        let ending = end_in_background(child, group, on_exit);
        Ok((
            Self {
                master: pair.master,
                group,
                reaped: Arc::clone(&ending.reaped),
                gone: Arc::clone(&ending.gone),
                hung_up: AtomicBool::new(false),
            },
            streams,
        ))
    }

    pub fn resize(&self, size: GridSize) {
        // A PTY whose child is gone refuses the resize; nothing to do then.
        let _ = self.master.resize(size.pty_size());
    }
}

fn builder(
    command: &ShellCommand,
    inherited: impl Iterator<Item = (OsString, OsString)>,
) -> CommandBuilder {
    let env = child_env(command, inherited);
    let mut builder = CommandBuilder::new(&command.program);
    builder.env_clear();
    builder.args(&command.args);
    builder.cwd(working_directory(command, &env));
    for (name, value) in env {
        builder.env(name, value);
    }
    builder
}

/// Where the program starts, always a directory that exists: the one the
/// command asks for, else the user's home, else the root. Left to itself the
/// PTY library uses `HOME` unchecked, and a home that is gone then keeps the
/// terminal from starting with an error that names the program.
fn working_directory(command: &ShellCommand, env: &[(OsString, OsString)]) -> PathBuf {
    let home = env
        .iter()
        .find(|(name, _)| name == "HOME")
        .map(|(_, home)| PathBuf::from(home));
    [command.cwd.clone(), home]
        .into_iter()
        .flatten()
        .find(|directory| directory.is_dir())
        .unwrap_or_else(|| PathBuf::from(ROOT))
}

struct Ending {
    reaped: Arc<AtomicBool>,
    gone: Arc<AtomicBool>,
}

/// Waits for the child, so it does not stay around as a zombie, and then
/// for everything else in its process group: what the child left behind is
/// hung up on and, if it ignores that, killed. `on_exit` runs when nothing
/// is left.
fn end_in_background(
    mut child: Box<dyn Child + Send + Sync>,
    group: Option<Pid>,
    on_exit: Box<dyn FnOnce() + Send>,
) -> Ending {
    let ending = Ending {
        reaped: Arc::new(AtomicBool::new(false)),
        gone: Arc::new(AtomicBool::new(false)),
    };
    thread::spawn({
        let (reaped, gone) = (Arc::clone(&ending.reaped), Arc::clone(&ending.gone));
        move || {
            let _ = child.wait();
            reaped.store(true, Ordering::Release);
            if let Some(group) = group {
                end_group(group);
            }
            // In this order: once it says "gone", `on_exit` has already run.
            on_exit();
            gone.store(true, Ordering::Release);
        }
    });
    ending
}

/// A group has members for as long as signalling it finds someone.
fn group_is_empty(group: Pid) -> bool {
    test_kill_process_group(group).is_err_and(|error| error == Errno::SRCH)
}

fn wait_for_empty_group(group: Pid, patience: Duration) -> bool {
    let deadline = Instant::now() + patience;
    while !group_is_empty(group) {
        if Instant::now() >= deadline {
            return false;
        }
        thread::sleep(REAP_POLL);
    }
    true
}

/// Hangup, a grace period, then a kill, for whatever is left in the group.
/// Processes that moved to a session of their own are out of reach.
fn end_group(group: Pid) {
    if group_is_empty(group) {
        return;
    }
    let _ = kill_process_group(group, Signal::HUP);
    if !wait_for_empty_group(group, HANGUP_GRACE) {
        let _ = kill_process_group(group, Signal::KILL);
        wait_for_empty_group(group, KILL_PATIENCE);
    }
}

impl PtyProcess {
    pub fn process_id(&self) -> Option<u32> {
        self.group
            .map(|group| group.as_raw_nonzero().get().unsigned_abs())
    }

    /// Whether the program and everything else in its group have ended.
    pub fn is_gone(&self) -> bool {
        self.gone.load(Ordering::Acquire)
    }

    /// Hangup for the whole group, then a kill if the program itself ignores
    /// it. What the program leaves behind is dealt with when it ends. Asking
    /// twice, or once all is gone, does nothing.
    pub fn hang_up(&self) {
        let Some(group) = self.group else {
            return;
        };
        if self.is_gone() || self.hung_up.swap(true, Ordering::AcqRel) {
            return;
        }
        if !self.reaped.load(Ordering::Acquire) {
            let _ = kill_process_group(group, Signal::HUP);
        }
        let reaped = Arc::clone(&self.reaped);
        thread::spawn(move || {
            let deadline = Instant::now() + HANGUP_GRACE;
            while !reaped.load(Ordering::Acquire) {
                if Instant::now() >= deadline {
                    let _ = kill_process_group(group, Signal::KILL);
                    return;
                }
                thread::sleep(REAP_POLL);
            }
        });
    }
}

impl Drop for PtyProcess {
    fn drop(&mut self) {
        self.hang_up();
    }
}
