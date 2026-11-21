use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

use lithemux_app::terminal_view::{GridSize, GridSnapshot, ShellCommand, TerminalSession};

const PATIENCE: Duration = Duration::from_secs(10);

pub const SIZE: GridSize = GridSize {
    columns: 80,
    lines: 24,
    cell_width: 8,
    cell_height: 16,
};

pub type Wakes = async_channel::Receiver<()>;

/// Starts the fake program in the given mode, on a real PTY.
pub fn start(mode: &str) -> TerminalSession {
    start_with_wakes(mode).0
}

pub fn start_with_wakes(mode: &str) -> (TerminalSession, Wakes) {
    let fakes = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fakes");
    let mut command = ShellCommand::new(fakes.join("fake-program"));
    command.args.push(mode.to_string());
    // Nothing from the machine: no profile, no user PATH, only the fakes
    // and the system tools the fake script calls.
    command.inherit_env = false;
    let path = format!("{}:/usr/bin:/bin", fakes.display());
    command.env.push(("PATH".to_string(), path));
    TerminalSession::spawn(&command, SIZE).unwrap()
}

/// Waits until `done` holds; panics with `what` after the patience runs out.
pub fn wait_until(what: &str, mut done: impl FnMut() -> bool) {
    let deadline = Instant::now() + PATIENCE;
    while !done() {
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        thread::sleep(Duration::from_millis(10));
    }
}

pub fn text(grid: &GridSnapshot) -> String {
    grid.rows
        .iter()
        .map(|runs| {
            let mut line = String::new();
            for run in runs {
                while line.chars().count() < run.column {
                    line.push(' ');
                }
                line.push_str(&run.text);
            }
            line
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Waits until the screen shows `needle` and returns the screen text.
pub fn wait_for(session: &TerminalSession, needle: &str) -> String {
    let deadline = Instant::now() + PATIENCE;
    loop {
        let screen = text(&session.snapshot());
        if screen.contains(needle) {
            return screen;
        }
        assert!(
            Instant::now() < deadline,
            "never saw {needle:?}; the screen shows:\n{screen}"
        );
        thread::sleep(Duration::from_millis(20));
    }
}
