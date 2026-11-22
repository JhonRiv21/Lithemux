//! The app around its features: `live` is what the sidebar and the
//! workspace share, `window` puts them side by side, `run` starts it all
//! and `quit_signals` ends it in order.

mod headless;
pub mod live;
mod quit_signals;
mod run;
pub mod tooltip;
pub mod window;

pub use run::run;
