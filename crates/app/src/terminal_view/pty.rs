//! The child process and its pseudo terminal.

use std::ffi::OsString;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, native_pty_system};
use rustix::process::{Pid, Signal, kill_process_group};

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
const ROOT: &str = "/";

pub struct PtyProcess {
    master: Box<dyn MasterPty + Send>,
    /// The child leads its own session, so its pid names its process group.
    group: Option<Pid>,
    /// Set once the child was waited for: its pid may belong to someone else.
    reaped: Arc<AtomicBool>,
}

pub struct PtyStreams {
    pub reader: Box<dyn Read + Send>,
    pub writer: Box<dyn Write + Send>,
}

impl PtyProcess {
    pub fn spawn(
        command: &ShellCommand,
        size: GridSize,
    ) -> Result<(Self, PtyStreams), TerminalError> {
        Self::spawn_in(command, size, std::env::vars_os())
    }

    /// `inherited` is the environment of the process that starts the
    /// terminal; the program gets what `child_env` makes of it.
    pub(super) fn spawn_in(
        command: &ShellCommand,
        size: GridSize,
        inherited: impl Iterator<Item = (OsString, OsString)>,
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
        Ok((
            Self {
                master: pair.master,
                group,
                reaped: reap_in_background(child),
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

/// Waits for the child so it does not stay around as a zombie; the flag is
/// set once its pid is free to be someone else's.
fn reap_in_background(mut child: Box<dyn Child + Send + Sync>) -> Arc<AtomicBool> {
    let reaped = Arc::new(AtomicBool::new(false));
    thread::spawn({
        let reaped = Arc::clone(&reaped);
        move || {
            let _ = child.wait();
            reaped.store(true, Ordering::Release);
        }
    });
    reaped
}

impl Drop for PtyProcess {
    // Hangup for the whole group, then a kill for whatever ignored it.
    fn drop(&mut self) {
        let Some(group) = self.group else {
            return;
        };
        if self.reaped.load(Ordering::Acquire) {
            return;
        }
        let _ = kill_process_group(group, Signal::HUP);
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
