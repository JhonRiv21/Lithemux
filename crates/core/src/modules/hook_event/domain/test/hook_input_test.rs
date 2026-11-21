use crate::modules::hook_event::domain::hook_event::HookEvent;
use crate::modules::hook_event::domain::hook_input::HookInput;

// PLAN §0.4: the hook reads Claude Code's documented stdin JSON
#[test]
fn the_documented_fields_are_read() {
    let input = HookInput::from_json(
        r#"{"session_id":"abc","transcript_path":"/x.jsonl","cwd":"/w","hook_event_name":"SessionStart","source":"resume"}"#,
    );
    assert_eq!(input.session_id.as_deref(), Some("abc"));
    assert_eq!(input.source.as_deref(), Some("resume"));
    assert_eq!(input.notification_type, None);
    assert_eq!(input.version, None);
}

// PLAN §0.4: a broken hook must never get in Claude's way
#[test]
fn unreadable_input_is_the_same_as_no_input() {
    for text in ["", "not json", "[1,2]", "{\"session_id\": 7}"] {
        assert_eq!(HookInput::from_json(text), HookInput::default(), "{text:?}");
    }
}

// PLAN §0.3: the five subcommands wired in the settings JSON
#[test]
fn the_five_subcommands_are_recognized() {
    for (subcommand, event) in [
        ("session-start", HookEvent::SessionStart),
        ("prompt-submit", HookEvent::PromptSubmit),
        ("stop", HookEvent::Stop),
        ("notification", HookEvent::Notification),
        ("session-end", HookEvent::SessionEnd),
    ] {
        assert_eq!(HookEvent::parse(subcommand), Some(event));
        assert_eq!(event.as_str(), subcommand);
    }
    assert_eq!(HookEvent::parse("pre-tool-use"), None);
}
