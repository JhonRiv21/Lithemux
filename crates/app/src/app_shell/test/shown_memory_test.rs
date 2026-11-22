//! PLAN §3.3 and correction round of 3A (10): «suaviza la cifra, que
//! saltaba de 95 a 175 MB en segundos».

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::app_shell::shown_memory::{ShownMemory, WINDOW};

const MB: u64 = 1024 * 1024;

fn one(id: &str, bytes: u64) -> HashMap<String, u64> {
    HashMap::from([(id.to_string(), bytes)])
}

#[test]
fn a_session_shows_the_highest_figure_of_the_last_minute() {
    let mut shown = ShownMemory::default();
    let start = Instant::now();
    let at = |seconds| start + Duration::from_secs(seconds);
    assert_eq!(shown.record(&one("a", 95 * MB), at(0))["a"], 95 * MB);
    assert_eq!(shown.record(&one("a", 175 * MB), at(3))["a"], 175 * MB);
    // The drop right after a peak is not shown: nothing to paint again.
    assert_eq!(shown.record(&one("a", 95 * MB), at(6))["a"], 175 * MB);
    assert_eq!(shown.record(&one("a", 96 * MB), at(60))["a"], 175 * MB);
}

#[test]
fn a_peak_older_than_a_minute_no_longer_counts() {
    let mut shown = ShownMemory::default();
    let start = Instant::now();
    shown.record(&one("a", 175 * MB), start);
    let later = start + WINDOW;
    assert_eq!(shown.record(&one("a", 95 * MB), later)["a"], 95 * MB);
}

#[test]
fn a_figure_is_rounded_up_to_the_megabyte() {
    let mut shown = ShownMemory::default();
    assert_eq!(
        shown.record(&one("a", 95 * MB + 1), Instant::now())["a"],
        96 * MB
    );
}

// A session that was not measured (its program is gone) is not shown, and
// starts from nothing when it runs again.
#[test]
fn a_session_no_longer_measured_is_forgotten() {
    let mut shown = ShownMemory::default();
    let start = Instant::now();
    shown.record(&one("a", 175 * MB), start);
    let without = shown.record(&HashMap::new(), start + Duration::from_secs(3));
    assert!(without.is_empty());
    let again = shown.record(&one("a", 20 * MB), start + Duration::from_secs(6));
    assert_eq!(again["a"], 20 * MB);
}
