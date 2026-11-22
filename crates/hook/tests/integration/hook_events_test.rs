use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixListener;

use lithemux_core::modules::session::State;
use lithemux_core::platform::paths::home::current_user;
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

// PLAN §0.4 table + §2.0 bug 4: prompt-submit, notification, post-tool-use, stop, session-end
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
        ("post-tool-use", "post_tool_use.json", State::Working),
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

// PLAN §2.0 bug 3: a compaction in the middle of a turn keeps the turn
#[test]
fn a_compaction_keeps_the_session_working_and_prints_the_context() {
    let seeded = SeededHome::new();
    seeded
        .hook("prompt-submit")
        .write_stdin(fixture("user_prompt_submit.json"))
        .assert()
        .success();

    let output = seeded
        .hook("session-start")
        .write_stdin(fixture("session_start_compact.json"))
        .output()
        .unwrap();

    assert_eq!(seeded.executor_now().state, State::Working);
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["hookSpecificOutput"]["hookEventName"], "SessionStart");
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

// PLAN §2.2: the card's preview and its unread count come from documented
// fields of the hooks' stdin, written by the real hook binary.
#[test]
fn the_hooks_leave_the_preview_and_the_unread_count_on_the_card() {
    let seeded = SeededHome::new();
    let fire = |event: &str, stdin: &str| {
        seeded
            .hook(event)
            .write_stdin(fixture(stdin))
            .assert()
            .success();
        seeded.executor_now()
    };

    let asked = fire("prompt-submit", "user_prompt_submit.json");
    assert_eq!(
        asked.preview.as_deref(),
        Some("Write a function to calculate the factorial of a number")
    );
    assert_eq!(asked.unread, 0);

    let blocked = fire("notification", "notification_permission.json");
    assert_eq!(
        blocked.preview.as_deref(),
        Some("Claude needs your permission")
    );
    assert_eq!(blocked.unread, 1);

    let answered = fire("stop", "stop_with_last_message.json");
    assert_eq!(
        answered.preview.as_deref(),
        Some("I've completed the refactoring. Here's a summary of what changed.")
    );
    assert_eq!(answered.unread, 2);

    // A Stop without the last message keeps the preview and still counts.
    let again = fire("stop", "stop.json");
    assert_eq!(again.preview, answered.preview);
    assert_eq!(again.unread, 3);
}

// Audit of block 3 (2): the hook binary finds the app's socket by itself
// when the home's path is too long to hold it.
#[test]
fn the_hook_reaches_an_app_whose_home_is_too_long_for_a_socket() {
    let seeded = SeededHome::with_a_long_home();
    let socket = seeded.home.app_socket();
    assert!(socket.is_outside_the_home());
    socket.prepare(current_user()).unwrap();
    let listener = UnixListener::bind(socket.path()).unwrap();

    seeded
        .hook("stop")
        .write_stdin(fixture("stop.json"))
        .assert()
        .success();

    let (stream, _) = listener.accept().unwrap();
    let mut line = String::new();
    BufReader::new(stream).read_line(&mut line).unwrap();
    assert!(line.contains(&seeded.executor.id), "{line}");
    std::fs::remove_file(socket.path()).unwrap();
}
