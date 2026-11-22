//! The settings window (PLAN §2.6). `form` is what it shows, without gpui;
//! `view` paints it and stores each change through `core`, applied at once.

pub mod form;
mod view;

#[cfg(test)]
mod test;

pub use view::SettingsWindow;
