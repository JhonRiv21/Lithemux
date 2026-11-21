use crate::modules::hook_event::domain::hook_event::HookEvent;
use crate::modules::hook_event::domain::hook_input::HookInput;
use crate::modules::hook_event::domain::transition::transition;
use crate::modules::session::State;

fn notification(kind: &str) -> HookInput {
    HookInput {
        notification_type: Some(kind.to_string()),
        ..HookInput::default()
    }
}

// PLAN §0.4 table: session-start
#[test]
fn session_start_leaves_the_session_idle_and_counts_as_activity() {
    let change = transition(HookEvent::SessionStart, &HookInput::default()).unwrap();
    assert_eq!(change.state, State::Idle);
    assert!(change.touches_activity);
    assert_eq!(change.preserves, None);
}

// PLAN §0.4 table: prompt-submit
#[test]
fn prompt_submit_marks_the_session_working() {
    let change = transition(HookEvent::PromptSubmit, &HookInput::default()).unwrap();
    assert_eq!(change.state, State::Working);
    assert!(change.touches_activity);
}

// PLAN §0.4 table: stop
#[test]
fn stop_returns_the_session_to_idle() {
    let change = transition(HookEvent::Stop, &HookInput::default()).unwrap();
    assert_eq!(change.state, State::Idle);
    assert!(change.touches_activity);
}

// PLAN §0.4 table: notification
#[test]
fn a_permission_request_marks_the_session_waiting() {
    for kind in [
        "permission_prompt",
        "elicitation_dialog",
        "elicitation_url_dialog",
    ] {
        let change = transition(HookEvent::Notification, &notification(kind)).unwrap();
        assert_eq!(change.state, State::Waiting, "{kind}");
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
    let change = transition(HookEvent::SessionEnd, &HookInput::default()).unwrap();
    assert_eq!(change.state, State::Ended);
    assert_eq!(change.preserves, Some(State::Hibernated));
}
