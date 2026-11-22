//! The workspace: up to four panes in a split tree, each with tabs. The
//! layout itself is `core`'s (`modules::workspace`), kept in the database.
//! `view` is the entity, `changes` what the user does to it and `shortcuts`
//! the keyboard; `start_policy` and `read_policy` are its decisions, without
//! gpui; `split_view`, `pane_view`, `tab_view` and `tab_drag` paint it.

mod changes;
mod pane_view;
pub mod read_policy;
mod shortcuts;
mod split_view;
pub mod start_policy;
mod tab_drag;
mod tab_view;
mod view;

#[cfg(test)]
mod test;

pub use shortcuts::bind_keys;
pub use view::Workspace;
