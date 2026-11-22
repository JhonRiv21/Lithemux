//! The app around its features: `live` is what the sidebar and the
//! workspace share, `window` puts them side by side, `run` starts it all
//! and `quit_signals` ends it in order.

mod headless;
pub mod live;
mod menus;
mod preferences;
mod quit_signals;
mod run;
pub mod title_bar;
pub mod tooltip;
pub mod window;

pub use run::run;
