use std::ffi::OsStr;

use crate::terminal_view::ime_log::{is_switched_on, kind_of_key, size_of};

// Audit of the §2.0 (I6): only `LITHEMUX_IME_LOG=1` turns the trace on
#[test]
fn the_trace_is_on_only_with_the_exact_value_one() {
    assert!(is_switched_on(Some(OsStr::new("1"))));
    for other in ["0", "", "true", "yes", "11", " 1"] {
        assert!(!is_switched_on(Some(OsStr::new(other))), "{other:?}");
    }
    assert!(!is_switched_on(None));
}

// Audit of the §2.0 (I6): what was typed never reaches the trace
#[test]
fn a_text_is_described_by_its_size_and_nothing_else() {
    let secret = "hunter2-ñ";
    let described = size_of(secret);

    assert_eq!(described, "9 chars / 10 bytes");
    // Nothing but two numbers and fixed words: no room for what was typed.
    let without_numbers: String = described.chars().filter(|c| !c.is_ascii_digit()).collect();
    assert_eq!(without_numbers, " chars /  bytes");
    // An accented letter and a dead key plus a letter differ only in size.
    assert_eq!(size_of("á"), "1 chars / 2 bytes");
    assert_eq!(size_of("´"), "1 chars / 2 bytes");
    assert_eq!(size_of("a"), "1 chars / 1 bytes");
}

// Audit of the delta (point 10): a key is named by its kind; control with
// a letter produces no text and is still not a dead key
#[test]
fn a_key_is_described_by_its_kind_and_never_by_its_name() {
    assert_eq!(kind_of_key("a", true, false), "character");
    assert_eq!(kind_of_key("´", false, false), "dead key");
    assert_eq!(kind_of_key("c", false, true), "modified key");
    assert_eq!(kind_of_key("enter", false, false), "named key");
    assert_eq!(kind_of_key("left", false, true), "named key");
}
