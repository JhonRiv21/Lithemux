use std::time::Duration;

use lithemux_core::modules::launch::{ShellMode, inspect_claude, resolve_claude};

use crate::support::fake_shell_probe::FakeShellProbe;
use crate::support::in_memory_store::InMemoryStore;

const LOCATE: &str = "command -v claude";
const NATIVE: &str = "/Users/x/.local/bin/claude";
const CASK: &str = "/opt/homebrew/bin/claude";

// Audit M2: the lookups of one launch share four seconds of waiting
#[test]
fn a_shell_that_does_not_answer_leaves_the_fallback_one_second() {
    let probe = FakeShellProbe::new()
        .executable(CASK)
        .answers(ShellMode::Login, LOCATE, CASK);

    resolve_claude(&InMemoryStore::new(), &probe).unwrap();

    let waits: Vec<Duration> = probe.calls.borrow().iter().map(|call| call.2).collect();
    assert_eq!(waits, [Duration::from_secs(3), Duration::from_secs(1)]);
}

// Audit M2: and so do the lookups of one `doctor`
#[test]
fn doctor_stops_asking_a_shell_that_used_up_its_wait() {
    let probe = FakeShellProbe::new();

    let inspection = inspect_claude(&InMemoryStore::new(), &probe);

    assert!(inspection.chosen.is_err());
    assert!(inspection.installations.is_empty());
    let asked: Vec<(ShellMode, Duration)> = probe
        .calls
        .borrow()
        .iter()
        .map(|call| (call.0, call.2))
        .collect();
    assert_eq!(
        asked,
        [
            (ShellMode::LoginInteractive, Duration::from_secs(3)),
            (ShellMode::Login, Duration::from_secs(1)),
        ]
    );
}

// Audit M2: a shell that answers is not charged, so `doctor` still lists
#[test]
fn a_shell_that_answers_keeps_its_whole_wait_for_the_next_question() {
    let probe = FakeShellProbe::new()
        .executable(NATIVE)
        .answers(ShellMode::LoginInteractive, LOCATE, NATIVE)
        .answers(ShellMode::LoginInteractive, "which -a claude", NATIVE);

    inspect_claude(&InMemoryStore::new(), &probe);

    let waits: Vec<Duration> = probe.calls.borrow().iter().map(|call| call.2).collect();
    assert_eq!(waits, [Duration::from_secs(3), Duration::from_secs(3)]);
}

// Audit of the delta (M2): a shell that fails at once has used no time, so
// the fallback keeps its whole wait
#[test]
fn a_shell_that_fails_at_once_leaves_the_fallback_its_whole_wait() {
    let probe = FakeShellProbe::new()
        .executable(CASK)
        .taking(ShellMode::LoginInteractive, Duration::from_millis(20))
        .answers(ShellMode::Login, LOCATE, CASK);

    resolve_claude(&InMemoryStore::new(), &probe).unwrap();

    let waits: Vec<Duration> = probe.calls.borrow().iter().map(|call| call.2).collect();
    assert_eq!(waits, [Duration::from_secs(3), Duration::from_secs(3)]);
}

// Audit of the delta (M2): a slow shell that answers is charged what it
// took, so `doctor` as a whole stays within its four seconds
#[test]
fn a_slow_shell_that_answers_is_charged_the_time_it_took() {
    let probe = FakeShellProbe::new()
        .executable(NATIVE)
        .taking(ShellMode::LoginInteractive, Duration::from_millis(2500))
        .answers(ShellMode::LoginInteractive, LOCATE, NATIVE)
        .answers(ShellMode::LoginInteractive, "which -a claude", NATIVE);

    inspect_claude(&InMemoryStore::new(), &probe);

    let waits: Vec<Duration> = probe.calls.borrow().iter().map(|call| call.2).collect();
    assert_eq!(waits, [Duration::from_secs(3), Duration::from_millis(1500)]);
}
