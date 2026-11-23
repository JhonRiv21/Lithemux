//! The window that imports cmux's sessions (PLAN §3.4). State, without
//! gpui: `state`. Actions: `actions` reads cmux's file and imports through
//! the use cases of `core`. View: `view`, painted with `rows_view`.

pub mod actions;
mod rows_view;
pub mod state;
mod view;

#[cfg(test)]
mod test;

pub use view::ImportWindow;
