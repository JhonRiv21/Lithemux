use crate::modules::hook_event::domain::hook_event::HookEvent;
use crate::modules::hook_event::domain::hook_input::HookInput;
use crate::modules::hook_event::domain::transition::{Transition, transition};
use crate::modules::session::State;

fn notification(kind: &str) -> HookInput {
    HookInput {
        notification_type: Some(kind.to_string()),
        ..HookInput::default()
    }
}

fn session_start(source: &str) -> HookInput {
    HookInput {
        source: Some(source.to_string()),
        ..HookInput::default()
    }
}

/// What a running session reports: it applies unless the app has
/// hibernated the session meanwhile (audit of block 4).
fn reports(state: State, touches_activity: bool) -> Option<Transition> {
    Some(Transition::Set {
        state,
        touches_activity,
        preserves: Some(State::Hibernated),
    })
}

fn sets(state: State, touches_activity: bool) -> Option<Transition> {
    Some(Transition::Set {
        state,
        touches_activity,
        preserves: None,
    })
}

// PLAN §0.4 table: session-start
#[test]
fn session_start_leaves_the_session_idle_and_counts_as_activity() {
    for source in ["startup", "resume", "clear", "fork"] {
        assert_eq!(
            transition(HookEvent::SessionStart, &session_start(source)),
            sets(State::Idle, true),
            "{source}"
        );
    }
    assert_eq!(
        transition(HookEvent::SessionStart, &HookInput::default()),
        sets(State::Idle, true)
    );
}

// PLAN §2.0 bug 3: a compaction is not a new start
#[test]
fn a_compaction_only_counts_as_activity() {
    assert_eq!(
        transition(HookEvent::SessionStart, &session_start("compact")),
        Some(Transition::Activity)
    );
}

// PLAN §0.4 table: prompt-submit
#[test]
fn prompt_submit_marks_the_session_working() {
    assert_eq!(
        transition(HookEvent::PromptSubmit, &HookInput::default()),
        reports(State::Working, true)
    );
}

// PLAN §0.4 table: stop
#[test]
fn stop_returns_the_session_to_idle() {
    assert_eq!(
        transition(HookEvent::Stop, &HookInput::default()),
        reports(State::Idle, true)
    );
}

// PLAN §2.0 bug 4: one transition and nothing else
#[test]
fn a_finished_tool_only_moves_waiting_to_working() {
    for event in [HookEvent::PostToolUse, HookEvent::PostToolUseFailure] {
        assert_eq!(
            transition(event, &HookInput::default()),
            Some(Transition::Move {
                from: State::Waiting,
                to: State::Working,
            })
        );
    }
}

// PLAN §0.4 table: notification
#[test]
fn a_permission_request_marks_the_session_waiting() {
    for kind in [
        "permission_prompt",
        "elicitation_dialog",
        "elicitation_url_dialog",
    ] {
        assert_eq!(
            transition(HookEvent::Notification, &notification(kind)),
            reports(State::Waiting, false),
            "{kind}"
        );
    }
}

// docs/decisiones.md: idle_prompt would break hibernation. It never puts a
// session in waiting, nor touches one that is in any other state: since
// block 4 it only moves one out of waiting (see the test below).
#[test]
fn an_idle_prompt_does_not_mark_the_session_waiting() {
    let change = transition(HookEvent::Notification, &notification("idle_prompt"));
    assert!(
        matches!(change, Some(Transition::Move { from: State::Waiting, to }) if to != State::Waiting),
        "{change:?}"
    );
}

// PLAN §0.4 table: other notifications change nothing
#[test]
fn unrelated_notifications_change_nothing() {
    assert_eq!(
        transition(HookEvent::Notification, &notification("auth_success")),
        None
    );
    assert_eq!(
        transition(HookEvent::Notification, &HookInput::default()),
        None
    );
}

// PLAN §0.4 table: session-end does not overwrite a deliberate hibernation
#[test]
fn session_end_marks_ended_but_preserves_hibernated() {
    assert_eq!(
        transition(HookEvent::SessionEnd, &HookInput::default()),
        Some(Transition::Set {
            state: State::Ended,
            touches_activity: false,
            preserves: Some(State::Hibernated),
        })
    );
}

// Block 4, part 0 (5): nothing documented fires when the user dismisses a
// permission prompt. The idle reminder, 60 s later, says Claude is idle:
// a session still marked as waiting is not waiting any more.
#[test]
fn the_idle_reminder_takes_a_session_out_of_waiting() {
    assert_eq!(
        transition(HookEvent::Notification, &notification("idle_prompt")),
        Some(Transition::Move {
            from: State::Waiting,
            to: State::Idle,
        })
    );
}

// Audit of block 4 (9): the app writes `hibernated` and then tells the
// program to leave. Whatever that program still reports on its way out is
// late news: only a new start of the session takes it out of hibernation.
#[test]
fn only_a_session_start_takes_a_session_out_of_hibernation() {
    let late = [
        (HookEvent::Stop, HookInput::default()),
        (HookEvent::PromptSubmit, HookInput::default()),
        (HookEvent::Notification, notification("permission_prompt")),
        (HookEvent::SessionEnd, HookInput::default()),
    ];
    for (event, input) in late {
        match transition(event, &input) {
            Some(Transition::Set { preserves, .. }) => {
                assert_eq!(preserves, Some(State::Hibernated), "{event:?}");
            }
            other => panic!("{event:?}: {other:?}"),
        }
    }
    for event in [HookEvent::PostToolUse, HookEvent::PostToolUseFailure] {
        assert!(matches!(
            transition(event, &HookInput::default()),
            Some(Transition::Move {
                from: State::Waiting,
                ..
            })
        ));
    }
    // Waking: the resumed `claude` reports its start, and that counts.
    assert_eq!(
        transition(HookEvent::SessionStart, &session_start("resume")),
        sets(State::Idle, true)
    );
}
