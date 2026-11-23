//! The app around its features: `live` is what the sidebar and the
//! workspace share, `window` puts them side by side, `run` starts it all
//! and `quit_signals` ends it in order.

mod claude_update;
pub mod elsewhere;
mod headless;
pub mod live;
mod menus;
mod palette_flow;
mod preferences;
mod quit_signals;
mod refresh;
mod run;
mod session_keys;
mod shown_memory;
pub mod title_bar;
pub mod tooltip;
pub mod window;

pub use run::run;

#[cfg(test)]
mod test;
