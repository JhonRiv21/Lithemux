//! A running terminal: the emulator grid plus the process that feeds it.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;

use alacritty_terminal::grid::{Dimensions, Scroll};
use alacritty_terminal::index::{Column, Line, Point, Side};
use alacritty_terminal::selection::{Selection, SelectionType};
use alacritty_terminal::sync::FairMutex;
use alacritty_terminal::term::{Term, TermMode, viewport_to_point};

use super::emulator::{Emulator, TermDimensions};
use super::error::TerminalError;
use super::grid::{GridSnapshot, snapshot};
use super::keys::{KeyModes, KeyPress, key_bytes};
use super::paste::paste_bytes;
use super::pty::{GridSize, PtyProcess, PtyStreams, ShellCommand};
use super::pump::{end_stalled_syncs, read_output, write_input};

/// A one-column grid makes alacritty's reflow of wide characters run away.
const MIN_COLUMNS: u16 = 2;
const FOCUS_IN: &[u8] = b"\x1b[I";
const FOCUS_OUT: &[u8] = b"\x1b[O";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SessionModes {
    pub app_cursor: bool,
    pub bracketed_paste: bool,
    pub focus_events: bool,
    /// The wheel moves the cursor instead of the scrollback (full screen programs).
    pub wheel_as_arrows: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridPoint {
    pub line: usize,
    pub column: usize,
    /// The pointer is on the right half of the cell.
    pub right_half: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionKind {
    Cells,
    Words,
    Lines,
}

fn normalized(size: GridSize) -> GridSize {
    GridSize {
        columns: size.columns.max(MIN_COLUMNS),
        lines: size.lines.max(1),
        ..size
    }
}

pub struct TerminalSession {
    emulator: Arc<FairMutex<Emulator>>,
    input: mpsc::Sender<Vec<u8>>,
    pty: PtyProcess,
    exited: Arc<AtomicBool>,
    size: GridSize,
}

impl TerminalSession {
    /// The receiver gets a message whenever there is something new to paint.
    pub fn spawn(
        command: &ShellCommand,
        size: GridSize,
    ) -> Result<(Self, async_channel::Receiver<()>), TerminalError> {
        let size = normalized(size);
        let (pty, streams) = PtyProcess::spawn(command, size)?;
        let PtyStreams { reader, writer } = streams;
        let (input, pending_input) = mpsc::channel();
        let (wake, wakes) = async_channel::bounded(1);
        let (sync_deadline, sync_deadlines) = mpsc::channel();
        let emulator = Arc::new(FairMutex::new(Emulator::new(size, input.clone())));
        let exited = Arc::new(AtomicBool::new(false));

        thread::spawn(move || write_input(writer, pending_input));
        thread::spawn({
            let emulator = Arc::clone(&emulator);
            let wake = wake.clone();
            move || end_stalled_syncs(&sync_deadlines, &emulator, &wake)
        });
        thread::spawn({
            let emulator = Arc::clone(&emulator);
            let exited = Arc::clone(&exited);
            move || {
                read_output(reader, &emulator, &wake, &sync_deadline);
                exited.store(true, Ordering::Release);
                let _ = wake.try_send(());
            }
        });

        Ok((
            Self {
                emulator,
                input,
                pty,
                exited,
                size,
            },
            wakes,
        ))
    }

    pub fn write(&self, bytes: Vec<u8>) {
        if !bytes.is_empty() {
            let _ = self.input.send(bytes);
        }
    }

    pub fn size(&self) -> GridSize {
        self.size
    }

    /// Typed text or a paste, as the program asked to receive pastes.
    pub fn paste(&self, text: &str) {
        self.write(paste_bytes(text, self.modes().bracketed_paste));
    }

    /// Sends the key if it is one the program handles; text keys are not.
    pub fn press(&self, press: KeyPress<'_>) -> bool {
        let modes = KeyModes {
            app_cursor: self.modes().app_cursor,
        };
        let Some(bytes) = key_bytes(press, modes) else {
            return false;
        };
        self.write(bytes);
        true
    }

    /// Tells the program the view gained or lost focus, if it asked to know.
    pub fn report_focus(&self, focused: bool) {
        if self.modes().focus_events {
            let sequence = if focused { FOCUS_IN } else { FOCUS_OUT };
            self.write(sequence.to_vec());
        }
    }

    pub fn resize(&mut self, size: GridSize) {
        let size = normalized(size);
        if size == self.size {
            return;
        }
        self.size = size;
        self.emulator.lock().term.resize(TermDimensions::from(size));
        self.pty.resize(size);
    }

    pub fn has_exited(&self) -> bool {
        self.exited.load(Ordering::Acquire)
    }

    pub fn snapshot(&self) -> GridSnapshot {
        snapshot(&self.emulator.lock().term)
    }

    pub fn modes(&self) -> SessionModes {
        let mode = *self.emulator.lock().term.mode();
        SessionModes {
            app_cursor: mode.contains(TermMode::APP_CURSOR),
            bracketed_paste: mode.contains(TermMode::BRACKETED_PASTE),
            focus_events: mode.contains(TermMode::FOCUS_IN_OUT),
            wheel_as_arrows: mode.contains(TermMode::ALT_SCREEN | TermMode::ALTERNATE_SCROLL),
        }
    }

    /// Positive moves towards older output.
    pub fn scroll(&self, lines: i32) {
        self.emulator
            .lock()
            .term
            .scroll_display(Scroll::Delta(lines));
    }

    pub fn scroll_to_bottom(&self) {
        self.emulator.lock().term.scroll_display(Scroll::Bottom);
    }

    pub fn start_selection(&self, kind: SelectionKind, at: GridPoint) {
        let term = &mut self.emulator.lock().term;
        let (point, side) = locate(term, at);
        let kind = match kind {
            SelectionKind::Cells => SelectionType::Simple,
            SelectionKind::Words => SelectionType::Semantic,
            SelectionKind::Lines => SelectionType::Lines,
        };
        term.selection = Some(Selection::new(kind, point, side));
    }

    pub fn extend_selection(&self, to: GridPoint) {
        let term = &mut self.emulator.lock().term;
        let (point, side) = locate(term, to);
        if let Some(selection) = term.selection.as_mut() {
            selection.update(point, side);
        }
    }

    pub fn clear_selection(&self) {
        self.emulator.lock().term.selection = None;
    }

    pub fn selected_text(&self) -> Option<String> {
        self.emulator
            .lock()
            .term
            .selection_to_string()
            .filter(|text| !text.is_empty())
    }
}

fn locate<T>(term: &Term<T>, at: GridPoint) -> (Point, Side) {
    let line = at.line.min(term.screen_lines().saturating_sub(1));
    let column = at.column.min(term.columns().saturating_sub(1));
    let point = viewport_to_point(
        term.grid().display_offset(),
        Point::new(line, Column(column)),
    );
    let side = if at.right_half {
        Side::Right
    } else {
        Side::Left
    };
    (Point::new(Line(point.line.0), point.column), side)
}
