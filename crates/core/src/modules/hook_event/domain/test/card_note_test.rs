use crate::modules::hook_event::domain::card_note::card_note;
use crate::modules::hook_event::domain::hook_event::HookEvent;
use crate::modules::hook_event::domain::hook_input::HookInput;

// The stdin shapes are the examples of code.claude.com/docs/en/hooks.

// PLAN §2.2: the preview uses the `prompt` of UserPromptSubmit.
#[test]
fn a_prompt_is_previewed_and_is_not_unread() {
    let input = HookInput::from_json(
        r#"{"session_id":"abc123","hook_event_name":"UserPromptSubmit",
            "prompt":"Write a function to calculate the factorial of a number"}"#,
    );
    let note = card_note(HookEvent::PromptSubmit, &input).unwrap();
    assert_eq!(
        note.message,
        Some("Write a function to calculate the factorial of a number")
    );
    assert!(!note.asks_for_attention);
}

// PLAN §2.2: the preview uses the `message` of Notification.
#[test]
fn a_notification_is_previewed_and_is_unread() {
    let input = HookInput::from_json(
        r#"{"hook_event_name":"Notification","message":"Claude needs your permission",
            "notification_type":"permission_prompt"}"#,
    );
    let note = card_note(HookEvent::Notification, &input).unwrap();
    assert_eq!(note.message, Some("Claude needs your permission"));
    assert!(note.asks_for_attention);
}

// PLAN §2.2: the last message of Stop, when the documented input has it.
#[test]
fn the_end_of_a_turn_previews_claudes_last_message_and_is_unread() {
    let input = HookInput::from_json(
        r#"{"hook_event_name":"Stop","stop_hook_active":false,
            "last_assistant_message":"I've completed the refactoring."}"#,
    );
    let note = card_note(HookEvent::Stop, &input).unwrap();
    assert_eq!(note.message, Some("I've completed the refactoring."));
    assert!(note.asks_for_attention);
}

// PLAN §2.2: a version whose Stop has no last message still counts the turn.
#[test]
fn the_end_of_a_turn_without_a_last_message_is_unread_and_keeps_the_preview() {
    let input = HookInput::from_json(r#"{"hook_event_name":"Stop"}"#);
    let note = card_note(HookEvent::Stop, &input).unwrap();
    assert_eq!(note.message, None);
    assert!(note.asks_for_attention);
}

// PLAN §2.2: unread means activity while it was not seen; the idle reminder
// repeats a turn that already counted (D-37).
#[test]
fn the_idle_reminder_leaves_the_card_alone() {
    let input = HookInput::from_json(
        r#"{"message":"Claude is waiting for your input","notification_type":"idle_prompt"}"#,
    );
    assert_eq!(card_note(HookEvent::Notification, &input), None);
}

// PLAN §2.2
#[test]
fn the_other_events_leave_the_card_alone() {
    let input = HookInput::default();
    for event in [
        HookEvent::SessionStart,
        HookEvent::SessionEnd,
        HookEvent::PostToolUse,
        HookEvent::PostToolUseFailure,
    ] {
        assert_eq!(card_note(event, &input), None);
    }
}

// Audit of block 3 (8): only the notifications that ask the user for
// something count as unread. The types are those of the hooks reference.
#[test]
fn a_notification_that_asks_for_nothing_leaves_the_card_alone() {
    for kind in [
        "auth_success",
        "auth_storage_failure",
        "elicitation_complete",
        "elicitation_response",
        "agent_completed",
        "quota_auto_resume_fired",
        "quota_auto_resume_stale",
        "quota_auto_resume_disabled",
        "plugin_notification",
        "something_new",
    ] {
        let input = HookInput::from_json(&format!(
            r#"{{"message":"for your information","notification_type":"{kind}"}}"#
        ));
        assert_eq!(card_note(HookEvent::Notification, &input), None, "{kind}");
    }
    // Without a type there is no telling what it asks for.
    let untyped = HookInput::from_json(r#"{"message":"hello"}"#);
    assert_eq!(card_note(HookEvent::Notification, &untyped), None);
}

// Audit of block 3 (8)
#[test]
fn the_notifications_that_block_claude_on_the_user_count_as_unread() {
    for kind in [
        "permission_prompt",
        "elicitation_dialog",
        "elicitation_url_dialog",
    ] {
        let input = HookInput::from_json(&format!(
            r#"{{"message":"Claude needs you","notification_type":"{kind}"}}"#
        ));
        let note = card_note(HookEvent::Notification, &input).unwrap();
        assert_eq!(note.message, Some("Claude needs you"), "{kind}");
        assert!(note.asks_for_attention, "{kind}");
    }
}

// Block 4, part 0 (5): what Claude asks for is pending, not the last thing
// said: it must not stay on the card once it is answered or dropped.
#[test]
fn a_request_for_the_user_is_a_pending_notice_and_what_is_said_is_not() {
    let asks = HookInput::from_json(
        r#"{"message":"Claude needs your permission","notification_type":"permission_prompt"}"#,
    );
    assert!(
        card_note(HookEvent::Notification, &asks)
            .unwrap()
            .is_pending
    );
    let says = HookInput::from_json(r#"{"last_assistant_message":"Done."}"#);
    assert!(!card_note(HookEvent::Stop, &says).unwrap().is_pending);
    let prompt = HookInput::from_json(r#"{"prompt":"do it"}"#);
    assert!(
        !card_note(HookEvent::PromptSubmit, &prompt)
            .unwrap()
            .is_pending
    );
}
