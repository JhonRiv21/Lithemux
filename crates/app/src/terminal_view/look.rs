//! What every terminal of the app looks like, as the settings say: the
//! font, its size and how much history is kept. One for the whole app.

use std::sync::RwLock;
use std::sync::atomic::{AtomicU16, AtomicUsize, Ordering};

pub const DEFAULT_FONT_FAMILY: &str = "Menlo";
const DEFAULT_FONT_SIZE: u16 = 13;
const DEFAULT_SCROLLBACK: usize = 5_000;

static FONT_SIZE: AtomicU16 = AtomicU16::new(DEFAULT_FONT_SIZE);
static SCROLLBACK: AtomicUsize = AtomicUsize::new(DEFAULT_SCROLLBACK);
/// Empty stands for the default.
static FONT_FAMILY: RwLock<String> = RwLock::new(String::new());

/// Returns whether anything changed. The scrollback applies to the
/// terminals started from now on.
pub fn set_look(font_family: &str, font_size: u16, scrollback: usize) -> bool {
    let size_changed = FONT_SIZE.swap(font_size, Ordering::AcqRel) != font_size;
    let history_changed = SCROLLBACK.swap(scrollback, Ordering::AcqRel) != scrollback;
    let family_changed = match FONT_FAMILY.write() {
        Ok(mut current) if *current != font_family => {
            *current = font_family.to_string();
            true
        }
        _ => false,
    };
    size_changed || history_changed || family_changed
}

pub fn font_family() -> String {
    let chosen = FONT_FAMILY.read().map(|family| family.clone());
    match chosen {
        Ok(family) if !family.is_empty() => family,
        _ => DEFAULT_FONT_FAMILY.to_string(),
    }
}

pub fn font_size() -> f32 {
    f32::from(FONT_SIZE.load(Ordering::Acquire))
}

pub fn scrollback() -> usize {
    SCROLLBACK.load(Ordering::Acquire)
}
