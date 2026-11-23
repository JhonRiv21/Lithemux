//! The palette (cmd+p): find any session by name, slug, category or role,
//! and jump to it.
//!
//! State, without gpui: `ranking` decides the order, `state` keeps the
//! search and the selection, `entries` turns the board into what is
//! searched. View: `view`.

pub mod entries;
pub mod ranking;
pub mod state;
mod view;

#[cfg(test)]
mod test;

pub use view::{Palette, PaletteEvent};
