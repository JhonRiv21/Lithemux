use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixListener;

use lithemux_core::modules::session::State;
use serde_json::Value;

use crate::seeded_home::{SeededHome, fixture};

// PLAN §0.4 table + §0.5: session-start prints the context for Claude
#[test]
fn session_start_prints_the_additional_context_and_marks_idle() {
    let seeded = SeededHome::new();
    let output = seeded
        .hook("session-start")
        .write_stdin(fixture("session_start_startup.json"))
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["hookSpecificOutput"]["hookEventName"], "SessionStart");
    let context = json["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap();
    assert!(context.contains("category \"Demo\""));
    assert!(context.contains("This session's name: demo-ejecutora"));
    assert!(context.contains("Mother of the category: demo-planificadora"));
    assert!(context.contains("categories/demo/board.md"));
    assert!(context.chars().count() < 2000);

    let session = seeded.executor_now();
    assert_eq!(session.state, State::Idle);
    assert!(session.last_activity_at.is_some());
}

// docs/decisiones.md: source=clear with a new session_id leaves the new one stored
#[test]
fn session_start_after_clear_follows_the_new_conversation() {
    let seeded = SeededHome::new();
    let old = seeded.executor.claude_session_id.clone().unwrap();
    seeded
        .hook("session-start")
        .write_stdin(fixture("session_start_clear.json"))
        .assert()
        .success();

    assert_eq!(
        seeded.executor_now().claude_session_id.as_deref(),
        Some("11111111-2222-4333-8444-555555555555")
    );
    let log = std::fs::read_to_string(seeded.home.hook_log_path()).unwrap();
    assert!(log.contains(&old), "{log}");
    assert!(
        log.contains("11111111-2222-4333-8444-555555555555"),
        "{log}"
    );
}

// PLAN §0.4 table: prompt-submit, notification, stop, session-end
#[test]
fn a_whole_turn_moves_the_state_and_prints_nothing() {
    let seeded = SeededHome::new();
    for (subcommand, stdin, expected) in [
        ("prompt-submit", "user_prompt_submit.json", State::Working),
        (
            "notification",
            "notification_permission.json",
            State::Waiting,
        ),
        ("stop", "stop.json", State::Idle),
        ("session-end", "session_end.json", State::Ended),
    ] {
        let output = seeded
            .hook(subcommand)
            .write_stdin(fixture(stdin))
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(0), "{subcommand}");
        assert!(output.stdout.is_empty(), "{subcommand}");
        assert!(output.stderr.is_empty(), "{subcommand}");
        assert_eq!(seeded.executor_now().state, expected, "{subcommand}");
    }
}

// PLAN §0.4 table: the hook does not step on a deliberate hibernation
#[test]
fn session_end_leaves_a_hibernated_session_hibernated() {
    let seeded = SeededHome::new();
    seeded
        .db()
        .conn()
        .execute("UPDATE sessions SET state = 'hibernated'", [])
        .unwrap();
    seeded
        .hook("session-end")
        .write_stdin(fixture("session_end.json"))
        .assert()
        .success();
    assert_eq!(seeded.executor_now().state, State::Hibernated);
}

// PLAN §0.4: one JSON line to <home>/app.sock when an app is listening
#[test]
fn the_app_socket_receives_the_session_and_the_event() {
    let seeded = SeededHome::new();
    let listener = UnixListener::bind(seeded.home.app_socket_path()).unwrap();

    seeded
        .hook("stop")
        .write_stdin(fixture("stop.json"))
        .assert()
        .success();

    let (stream, _) = listener.accept().unwrap();
    let mut line = String::new();
    BufReader::new(stream).read_line(&mut line).unwrap();
    let message: Value = serde_json::from_str(&line).unwrap();
    assert_eq!(message["session_id"], seeded.executor.id.as_str());
    assert_eq!(message["event"], "stop");
}
