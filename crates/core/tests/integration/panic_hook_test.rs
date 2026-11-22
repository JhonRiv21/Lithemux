//! PLAN §3.6: a panic is written to the app's log. The panics happen in
//! this test binary run again, so the test itself does not panic.

use std::process::Stdio;
use std::sync::Arc;

use lithemux_core::platform::log::app_log::{AppLog, FILES_KEPT, MAX_BYTES};
use lithemux_core::platform::log::panic_hook::log_panics;

use crate::sandbox::this_test_binary;

const ROLE: &str = "LITHEMUX_TEST_PANIC_ROLE";
const LOG: &str = "LITHEMUX_TEST_PANIC_LOG";

/// What AppKit calls on the main thread is `extern "C"`: a panic there
/// cannot unwind, and the process aborts.
extern "C" fn called_by_the_system() {
    panic!("broken inside a callback of the system");
}

/// Not a test of its own: the body of the processes the tests start.
#[test]
fn panicker() {
    let (Ok(role), Ok(log)) = (
        std::env::var("LITHEMUX_TEST_PANIC_ROLE"),
        std::env::var("LITHEMUX_TEST_PANIC_LOG"),
    ) else {
        return;
    };
    log_panics(Arc::new(AppLog::new(log.into(), MAX_BYTES, FILES_KEPT)));
    if role == "abort" {
        called_by_the_system();
    }
    if role == "long" {
        // What slicing text or unwrapping an error can put in a message:
        // a piece of whatever was being handled.
        let handled = format!("{}SECRET-TAIL", "word ".repeat(60));
        panic!("byte index 400 is out of range of `{handled}`");
    }
    panic!("broken on purpose");
}

fn panic_as(role: &str) -> (std::process::ExitStatus, String) {
    let dir = tempfile::tempdir().unwrap();
    let log = dir.path().join("logs").join("app.log");
    let status = this_test_binary()
        .args(["--exact", "panic_hook_test::panicker"])
        .env(ROLE, role)
        .env(LOG, &log)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap();
    (status, std::fs::read_to_string(log).unwrap_or_default())
}

#[test]
fn a_panic_is_written_to_the_log_with_its_place() {
    let (status, log) = panic_as("unwind");
    assert!(!status.success());
    let line = log.lines().next().unwrap_or_default();
    assert!(line.contains(" ERROR panic in thread "), "{log}");
    assert!(line.contains("panic_hook_test.rs:"), "{log}");
    assert!(line.ends_with("broken on purpose"), "{log}");
}

// A panic that cannot unwind aborts the process; the hook has written
// before that.
#[test]
fn a_panic_that_aborts_the_process_is_written_too() {
    let (status, log) = panic_as("abort");
    assert!(!status.success());
    assert!(
        log.contains("broken inside a callback of the system"),
        "{log}"
    );
}

// Correction round of 3A (8): a panic's message can carry the text that
// was being handled; only its first 120 characters are kept.
#[test]
fn a_long_panic_message_is_cut_to_120_characters() {
    let (status, log) = panic_as("long");
    assert!(!status.success());
    let line = log.lines().next().unwrap_or_default();
    assert!(line.contains("panic_hook_test.rs:"), "{log}");
    assert!(line.contains("byte index 400 is out of range"), "{log}");
    assert!(!line.contains("SECRET-TAIL"), "{log}");
    let message = line
        .split_once("rs:")
        .map(|(_, rest)| rest)
        .unwrap_or_default();
    let message = message
        .split_once(": ")
        .map(|(_, said)| said)
        .unwrap_or_default();
    assert!(
        message.chars().count() <= 121,
        "{} chars: {message}",
        message.chars().count()
    );
}
