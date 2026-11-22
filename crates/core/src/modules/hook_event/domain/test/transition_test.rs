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
        sets(State::Working, true)
    );
}

// PLAN §0.4 table: stop
#[test]
fn stop_returns_the_session_to_idle() {
    assert_eq!(
        transition(HookEvent::Stop, &HookInput::default()),
        sets(State::Idle, true)
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
            sets(State::Waiting, false),
            "{kind}"
        );
    }
}

// docs/decisiones.md: idle_prompt would break hibernation
#[test]
fn an_idle_prompt_does_not_mark_the_session_waiting() {
    assert_eq!(
        transition(HookEvent::Notification, &notification("idle_prompt")),
        None
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
