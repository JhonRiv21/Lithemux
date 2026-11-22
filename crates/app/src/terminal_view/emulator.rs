//! The emulator state: the grid, the escape sequence parser and the answers
//! the program is owed.

use std::sync::Arc;
use std::sync::mpsc;
use std::time::Instant;

use alacritty_terminal::event::{Event, EventListener};
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::term::{Config, Term};
use alacritty_terminal::vte::ansi::{Processor, Rgb};

use super::look;
use super::palette;
use super::pty::GridSize;

pub(super) struct TermDimensions {
    columns: usize,
    lines: usize,
}

impl Dimensions for TermDimensions {
    fn total_lines(&self) -> usize {
        self.lines
    }

    fn screen_lines(&self) -> usize {
        self.lines
    }

    fn columns(&self) -> usize {
        self.columns
    }
}

impl From<GridSize> for TermDimensions {
    fn from(size: GridSize) -> Self {
        Self {
            columns: usize::from(size.columns),
            lines: usize::from(size.lines),
        }
    }
}

type ColorFormatter = Arc<dyn Fn(Rgb) -> String + Sync + Send + 'static>;

/// Answers the emulator needs to send back to the program.
pub(super) struct Replies {
    input: mpsc::Sender<Vec<u8>>,
    /// Answered by `Emulator`, which can read the colors the program set.
    color_requests: mpsc::Sender<(usize, ColorFormatter)>,
}

impl EventListener for Replies {
    fn send_event(&self, event: Event) {
        match event {
            Event::PtyWrite(text) => {
                let _ = self.input.send(text.into_bytes());
            }
            Event::ColorRequest(index, format) => {
                let _ = self.color_requests.send((index, format));
            }
            _ => {}
        }
    }
}

pub(super) struct Emulator {
    pub term: Term<Replies>,
    parser: Processor,
    color_requests: mpsc::Receiver<(usize, ColorFormatter)>,
    input: mpsc::Sender<Vec<u8>>,
}

impl Emulator {
    pub fn new(size: GridSize, input: mpsc::Sender<Vec<u8>>) -> Self {
        let (color_requests, pending_colors) = mpsc::channel();
        let config = Config {
            scrolling_history: look::scrollback(),
            ..Config::default()
        };
        let replies = Replies {
            input: input.clone(),
            color_requests,
        };
        Self {
            term: Term::new(config, &TermDimensions::from(size), replies),
            parser: Processor::new(),
            color_requests: pending_colors,
            input,
        }
    }

    /// Returns when a synchronized update left open must be given up on.
    pub fn feed(&mut self, bytes: &[u8]) -> Option<Instant> {
        self.parser.advance(&mut self.term, bytes);
        self.answer_color_requests();
        self.parser.sync_timeout().sync_timeout()
    }

    /// Shows what a synchronized update was holding back, if its time is up.
    pub fn end_sync_due_by(&mut self, now: Instant) -> bool {
        let overdue = self
            .parser
            .sync_timeout()
            .sync_timeout()
            .is_some_and(|deadline| deadline <= now);
        if overdue {
            self.parser.stop_sync(&mut self.term);
            self.answer_color_requests();
        }
        overdue
    }

    fn answer_color_requests(&mut self) {
        for (index, format) in self.color_requests.try_iter() {
            let color = self.term.colors()[index].unwrap_or_else(|| palette::default_color(index));
            let _ = self.input.send(format(color).into_bytes());
        }
    }
}
