//! The strip at the bottom of the window: the memory the sessions use
//! (PLAN §3.3) and the notice of a new Claude Code version (PLAN §3.2).
//! `memory_label` is how it writes memory, without gpui; `view` paints it.

pub mod memory_label;
mod view;

#[cfg(test)]
mod test;

pub use view::StatusBar;
