//! A terminal as a gpui view. `view`, `ime` and `paint` are the gpui side;
//! `session`, `emulator`, `pump`, `grid` and `pty` hold the state; `keys`,
//! `paste` and `composition` turn input into bytes.

mod composition;
mod emulator;
mod error;
mod font_probe;
mod grid;
mod ime;
mod keys;
mod paint;
mod palette;
mod paste;
mod pty;
mod pump;
mod session;
mod view;

#[cfg(test)]
mod test;

pub use error::TerminalError;
pub use font_probe::probe_text_rendering;
pub use grid::{CellStyle, CursorSnapshot, GridSnapshot, RunSnapshot};
pub use keys::{KeyModifiers, KeyPress};
pub use pty::{GridSize, ShellCommand};
pub use session::{GridPoint, SelectionKind, SessionModes, TerminalSession};
pub use view::TerminalView;
