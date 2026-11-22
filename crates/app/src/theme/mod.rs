//! The colors of the interface: a dark and a light palette, and which one
//! is in use. The whole app has one theme at a time.

mod palette;

#[cfg(test)]
mod test;

use std::sync::atomic::{AtomicBool, Ordering};

use lithemux_core::modules::settings::ThemeChoice;

pub use palette::{DARK, LIGHT, Palette};

static LIGHT_IS_IN_USE: AtomicBool = AtomicBool::new(false);

/// Whether the light palette applies: what the user chose or, for
/// «system», what macOS is set to.
pub fn is_light_for(choice: ThemeChoice, system_is_dark: bool) -> bool {
    match choice {
        ThemeChoice::Light => true,
        ThemeChoice::Dark => false,
        ThemeChoice::System => !system_is_dark,
    }
}

/// Returns whether it changed.
pub fn use_light(light: bool) -> bool {
    LIGHT_IS_IN_USE.swap(light, Ordering::AcqRel) != light
}

pub fn is_light() -> bool {
    LIGHT_IS_IN_USE.load(Ordering::Acquire)
}

/// The palette in use.
pub fn colors() -> &'static Palette {
    if is_light() { &LIGHT } else { &DARK }
}
