//! The sidebar: every session as a card, by category.
//!
//! State, without gpui: `tree` (what is shown), `status`, `cwd_label`,
//! `order` and `menu`. Actions: `actions` changes what is stored through
//! the use cases of `core`. View: `view` is the entity, with `form_flow`
//! and `deletion`; `group_view`, `card_view` and `overlay_view` paint it,
//! with `form`, `drag` and `status_text`.

pub mod actions;
mod card_view;
pub mod cwd_label;
mod deletion;
mod drag;
pub mod error;
mod form;
mod form_flow;
mod group_view;
pub mod menu;
pub mod order;
mod overlay_view;
pub mod port_chips;
pub mod status;
pub mod status_text;
pub mod tree;
mod view;

#[cfg(test)]
mod test;

pub use view::Sidebar;
