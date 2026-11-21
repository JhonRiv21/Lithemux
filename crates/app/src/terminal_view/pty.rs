//! The child process and its pseudo terminal.

use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use portable_pty::{CommandBuilder, MasterPty, PtySize, native_pty_system};
use rustix::process::{Pid, Signal, kill_process_group};

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
        let pair = native_pty_system()
            .openpty(size.pty_size())
            .map_err(|error| TerminalError::OpenPty(error.to_string()))?;

        let mut builder = CommandBuilder::new(&command.program);
        if !command.inherit_env {
            builder.env_clear();
        }
        builder.args(&command.args);
        if let Some(cwd) = &command.cwd {
            builder.cwd(cwd);
        }
        builder.env("TERM", "xterm-256color");
        builder.env("COLORTERM", "truecolor");
        builder.env("TERM_PROGRAM", "lithemux");
        for (name, value) in &command.env {
            builder.env(name, value);
        }

        let mut child =
            pair.slave
                .spawn_command(builder)
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
        let reaped = Arc::new(AtomicBool::new(false));
        // Reaps the child so it does not stay around as a zombie.
        thread::spawn({
            let reaped = Arc::clone(&reaped);
            move || {
                let _ = child.wait();
                reaped.store(true, Ordering::Release);
            }
        });
        Ok((
            Self {
                master: pair.master,
                group,
                reaped,
            },
            streams,
        ))
    }

    pub fn resize(&self, size: GridSize) {
        // A PTY whose child is gone refuses the resize; nothing to do then.
        let _ = self.master.resize(size.pty_size());
    }
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
