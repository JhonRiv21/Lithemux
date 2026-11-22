use lithemux_core::modules::hook_event::HookEvent;
use lithemux_core::modules::session::{State, apply_state};
use serde_json::Value;

use crate::support::hook_world::World;
use crate::support::sample_data::NOW;

// PLAN §0.4 table: session-start
#[test]
fn session_start_marks_idle_and_prints_the_context() {
    let world = World::new();
    let stdout = world
        .fire("session-start", r#"{"source":"startup"}"#, NOW)
        .unwrap();
    let json: Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["hookSpecificOutput"]["hookEventName"], "SessionStart");
    let context = json["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap();
    assert!(context.contains("This session's name: demo-ejecutora"));
    assert!(context.contains("Coordinator of the category: demo-planificadora"));

    let session = world.executor_now();
    assert_eq!(session.state, State::Idle);
    assert_eq!(session.last_activity_at, Some(NOW));
}

// PLAN §0.4 table: prompt-submit, stop
#[test]
fn a_turn_goes_working_then_idle_and_prints_nothing() {
    let world = World::new();
    assert_eq!(
        world.fire("prompt-submit", r#"{"prompt":"hola"}"#, NOW),
        None
    );
    assert_eq!(world.executor_now().state, State::Working);
    assert_eq!(world.executor_now().last_activity_at, Some(NOW));
    assert_eq!(world.fire("stop", "{}", NOW + 30), None);
    assert_eq!(world.executor_now().state, State::Idle);
    assert_eq!(world.executor_now().last_activity_at, Some(NOW + 30));
}

// PLAN §0.4 table: notification
#[test]
fn a_permission_prompt_marks_waiting_without_counting_as_activity() {
    let world = World::new();
    world.fire("prompt-submit", "{}", NOW);
    world.fire(
        "notification",
        r#"{"notification_type":"permission_prompt"}"#,
        NOW + 60,
    );
    let session = world.executor_now();
    assert_eq!(session.state, State::Waiting);
    assert_eq!(session.last_activity_at, Some(NOW));
}

// docs/decisiones.md: idle_prompt must not turn every idle session into waiting
#[test]
fn an_idle_prompt_keeps_the_session_idle() {
    let world = World::new();
    world.fire("stop", "{}", NOW);
    world.fire(
        "notification",
        r#"{"notification_type":"idle_prompt"}"#,
        NOW + 60,
    );
    assert_eq!(world.executor_now().state, State::Idle);
}

// PLAN §0.4 table: session-end
#[test]
fn session_end_marks_the_session_ended() {
    let world = World::new();
    world.fire("session-end", r#"{"reason":"prompt_input_exit"}"#, NOW);
    assert_eq!(world.executor_now().state, State::Ended);
}

// PLAN §0.4 table: the hook does not overwrite a deliberate hibernation
#[test]
fn session_end_does_not_overwrite_hibernated() {
    let world = World::new();
    apply_state(
        &world.store,
        &world.executor.id,
        State::Hibernated,
        None,
        None,
    )
    .unwrap();
    world.fire("session-end", "{}", NOW);
    assert_eq!(world.executor_now().state, State::Hibernated);
}

// docs/decisiones.md: a new session_id with source=clear is the one that stays
#[test]
fn session_start_after_clear_stores_the_new_conversation_id() {
    let world = World::new();
    let old = world.executor.claude_session_id.clone().unwrap();
    let new = "11111111-2222-4333-8444-555555555555";
    let stdin = format!(r#"{{"session_id":"{new}","source":"clear"}}"#);

    assert!(world.fire("session-start", &stdin, NOW).is_some());

    assert_eq!(world.executor_now().claude_session_id.as_deref(), Some(new));
    let log = world.log.lines.borrow();
    assert!(
        log.iter()
            .any(|line| line.contains(&old) && line.contains(new) && line.contains("clear")),
        "the change old -> new is logged: {log:?}"
    );
}

// docs/decisiones.md: any source follows the conversation, also resume and fork
#[test]
fn every_session_start_source_follows_the_conversation() {
    for source in ["startup", "resume", "clear", "compact", "fork"] {
        let world = World::new();
        let new = "aaaaaaaa-bbbb-4ccc-8ddd-eeeeeeeeeeee";
        let stdin = format!(r#"{{"session_id":"{new}","source":"{source}"}}"#);
        world.fire("session-start", &stdin, NOW);
        assert_eq!(
            world.executor_now().claude_session_id.as_deref(),
            Some(new),
            "{source}"
        );
    }
}

// docs/decisiones.md: the same id is not a change
#[test]
fn session_start_with_the_stored_id_logs_nothing() {
    let world = World::new();
    let same = world.executor.claude_session_id.clone().unwrap();
    world.fire(
        "session-start",
        &format!(r#"{{"session_id":"{same}"}}"#),
        NOW,
    );
    assert!(world.log.lines.borrow().is_empty());
}

// PLAN §0.4: session-start uses the version when the stdin brings one
#[test]
fn session_start_uses_the_version_when_present_and_keeps_it_otherwise() {
    let world = World::new();
    world.fire("session-start", r#"{"version":"2.1.330"}"#, NOW);
    assert_eq!(
        world.executor_now().claude_version.as_deref(),
        Some("2.1.330")
    );
    world.fire("session-start", "{}", NOW);
    assert_eq!(
        world.executor_now().claude_version.as_deref(),
        Some("2.1.330")
    );
}

// PLAN §0.4: after writing, the app is told which session changed
#[test]
fn every_event_is_announced_to_the_app() {
    let world = World::new();
    world.fire("stop", "{}", NOW);
    assert_eq!(
        *world.notifier.events.borrow(),
        vec![(world.executor.id.clone(), HookEvent::Stop)]
    );
}

// PLAN §0.4: errors go to the log, never to Claude
#[test]
fn an_unknown_event_only_leaves_a_log_line() {
    let world = World::new();
    assert_eq!(world.fire("pre-tool-use", "{}", NOW), None);
    assert_eq!(world.executor_now().state, State::New);
    assert_eq!(world.log.lines.borrow().len(), 1);
    assert!(world.notifier.events.borrow().is_empty());
}

// PLAN §0.4: garbage on stdin never breaks the hook
#[test]
fn garbage_on_stdin_still_updates_the_state() {
    let world = World::new();
    assert_eq!(world.fire("stop", "<<not json>>", NOW), None);
    assert_eq!(world.executor_now().state, State::Idle);
}

fn put_executor_in(world: &World, state: State, at: i64) {
    apply_state(&world.store, &world.executor.id, state, Some(at), None).unwrap();
}

// PLAN §2.0 bug 3: a compaction happens in the middle of a turn
#[test]
fn a_compaction_keeps_the_state_and_still_delivers_the_context() {
    for state in [State::Working, State::Waiting] {
        let world = World::new();
        put_executor_in(&world, state, NOW);

        let stdout = world.fire("session-start", r#"{"source":"compact"}"#, NOW + 60);

        let session = world.executor_now();
        assert_eq!(session.state, state);
        assert_eq!(session.last_activity_at, Some(NOW + 60));
        assert!(stdout.unwrap().contains("demo-ejecutora"));
    }
}

// PLAN §2.0 bug 4: approving the permission lets the tool run
#[test]
fn a_finished_tool_takes_a_waiting_session_back_to_working() {
    let world = World::new();
    put_executor_in(&world, State::Waiting, NOW);

    let stdout = world.fire("post-tool-use", "{}", NOW + 5);

    assert_eq!(stdout, None);
    assert_eq!(world.executor_now().state, State::Working);
}

// PLAN §2.0 bug 4: "sin tocar ningún otro estado"
#[test]
fn a_finished_tool_changes_no_state_other_than_waiting() {
    for state in [
        State::New,
        State::Working,
        State::Idle,
        State::Hibernated,
        State::Ended,
    ] {
        let world = World::new();
        put_executor_in(&world, state, NOW);

        world.fire("post-tool-use", "{}", NOW + 5);

        let session = world.executor_now();
        assert_eq!(session.state, state);
        assert_eq!(session.last_activity_at, Some(NOW));
        assert!(world.log.lines.borrow().is_empty());
    }
}

// docs/decisiones.md D-20: a tool that fails after its permission was
// granted also lets the turn go on
#[test]
fn a_failed_tool_takes_a_waiting_session_back_to_working_too() {
    let world = World::new();
    put_executor_in(&world, State::Waiting, NOW);

    let stdout = world.fire("post-tool-use-failure", "{}", NOW + 5);

    assert_eq!(stdout, None);
    assert_eq!(world.executor_now().state, State::Working);
    assert!(world.log.lines.borrow().is_empty());
}

// PLAN §2.2: the card's preview and unread count come from the hooks.
#[test]
fn a_turn_leaves_its_last_message_on_the_card_and_counts_as_unread() {
    let world = World::new();
    world.fire("prompt-submit", r#"{"prompt":"fix the\nlogin"}"#, NOW);
    let asked = world.executor_now();
    assert_eq!(
        (asked.preview.as_deref(), asked.unread),
        (Some("fix the login"), 0)
    );
    world.fire(
        "stop",
        r#"{"last_assistant_message":"Fixed. The token was read too late."}"#,
        NOW + 5,
    );
    let answered = world.executor_now();
    assert_eq!(
        (answered.preview.as_deref(), answered.unread),
        (Some("Fixed. The token was read too late."), 1)
    );
}

// PLAN §2.2; block 4, part 0 (5): the request is on the card while Claude
// waits, and counts once. The idle reminder is not counted again.
#[test]
fn a_permission_request_shows_on_the_card_while_it_is_pending() {
    let world = World::new();
    world.fire(
        "notification",
        r#"{"message":"Claude needs your permission to use Bash","notification_type":"permission_prompt"}"#,
        NOW,
    );
    let waiting = world.executor_now();
    assert_eq!(
        waiting.card_line(),
        Some("Claude needs your permission to use Bash")
    );
    assert_eq!(waiting.unread, 1);

    world.fire(
        "notification",
        r#"{"message":"Claude is waiting for your input","notification_type":"idle_prompt"}"#,
        NOW + 60,
    );
    let later = world.executor_now();
    assert_eq!(later.unread, 1);
    assert_ne!(later.card_line(), Some("Claude is waiting for your input"));
}

// Block 4, part 0 (5): what Claude asked for leaves the card when the wait
// ends, however it ends, and what was said last is back.
#[test]
fn a_permission_request_leaves_the_card_once_claude_no_longer_waits() {
    let ask = r#"{"message":"Claude needs your permission to use Bash","notification_type":"permission_prompt"}"#;
    let ways_out = [
        ("post-tool-use", "{}", State::Working),
        ("stop", "{}", State::Idle),
        (
            "prompt-submit",
            r#"{"prompt":"never mind"}"#,
            State::Working,
        ),
        ("session-end", "{}", State::Ended),
        // The user dismissed the prompt: nothing fires until the reminder.
        (
            "notification",
            r#"{"notification_type":"idle_prompt"}"#,
            State::Idle,
        ),
    ];
    for (event, stdin, state) in ways_out {
        let world = World::new();
        world.fire("stop", r#"{"last_assistant_message":"Let me check."}"#, NOW);
        world.fire("notification", ask, NOW + 1);
        let waiting = world.executor_now();
        assert_eq!(waiting.state, State::Waiting);
        assert_eq!(
            waiting.card_line(),
            Some("Claude needs your permission to use Bash")
        );
        assert_eq!(waiting.preview.as_deref(), Some("Let me check."));

        world.fire(event, stdin, NOW + 2);
        let after = world.executor_now();
        assert_eq!(after.state, state, "{event}");
        assert_eq!(after.notice, None, "{event}");
        assert_ne!(
            after.card_line(),
            Some("Claude needs your permission to use Bash"),
            "{event}"
        );
    }
}

// Block 4, part 0 (5): the reminder only takes a session out of waiting.
#[test]
fn the_idle_reminder_changes_nothing_of_a_session_that_is_not_waiting() {
    let world = World::new();
    world.fire("prompt-submit", r#"{"prompt":"go"}"#, NOW);
    world.fire(
        "notification",
        r#"{"notification_type":"idle_prompt"}"#,
        NOW + 60,
    );
    assert_eq!(world.executor_now().state, State::Working);
}

// Audit of block 4 (9): a session the app has just hibernated stays so,
// whatever its program still reports while it leaves; the start of its
// next run wakes it.
#[test]
fn late_news_of_a_hibernated_session_do_not_wake_it_and_its_next_start_does() {
    let world = World::new();
    world.fire("session-start", r#"{"source":"startup"}"#, NOW);
    apply_state(
        &world.store,
        &world.executor.id,
        State::Hibernated,
        None,
        None,
    )
    .unwrap();

    world.fire("stop", "{}", NOW + 1);
    world.fire("prompt-submit", r#"{"prompt":"late"}"#, NOW + 2);
    world.fire(
        "notification",
        r#"{"message":"late","notification_type":"permission_prompt"}"#,
        NOW + 3,
    );
    world.fire("session-end", "{}", NOW + 4);
    assert_eq!(world.executor_now().state, State::Hibernated);

    world.fire("session-start", r#"{"source":"resume"}"#, NOW + 10);
    assert_eq!(world.executor_now().state, State::Idle);
}
