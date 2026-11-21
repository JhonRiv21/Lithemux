use std::path::Path;

use serde_json::{Value, json};

use crate::modules::launch::domain::hooks_settings::hooks_settings_json;

// PLAN §0.3: the JSON of hooks, literally as specified
#[test]
fn the_settings_json_matches_the_plan() {
    let actual: Value =
        serde_json::from_str(&hooks_settings_json(Path::new("/abs/lithemux-hook"))).unwrap();
    let expected = json!({"hooks":{
        "SessionStart":[{"hooks":[{"type":"command","command":"/abs/lithemux-hook session-start","timeout":5}]}],
        "UserPromptSubmit":[{"hooks":[{"type":"command","command":"/abs/lithemux-hook prompt-submit","timeout":2}]}],
        "Stop":[{"hooks":[{"type":"command","command":"/abs/lithemux-hook stop","timeout":2}]}],
        "Notification":[{"hooks":[{"type":"command","command":"/abs/lithemux-hook notification","timeout":2}]}],
        "SessionEnd":[{"hooks":[{"type":"command","command":"/abs/lithemux-hook session-end","timeout":2}]}]}});
    assert_eq!(actual, expected);
}

// PLAN §0.3: only five events, nothing as frequent as PreToolUse
#[test]
fn only_five_events_and_no_pre_tool_use() {
    let settings: Value =
        serde_json::from_str(&hooks_settings_json(Path::new("/abs/lithemux-hook"))).unwrap();
    let hooks = settings["hooks"].as_object().unwrap();
    assert_eq!(hooks.len(), 5);
    assert!(!hooks.contains_key("PreToolUse"));
    assert!(!hooks.contains_key("PostToolUse"));
}

// PLAN §0.3: hook commands run through a shell, so a path with spaces must survive
#[test]
fn a_hook_path_with_spaces_is_quoted_for_the_shell() {
    let settings: Value = serde_json::from_str(&hooks_settings_json(Path::new(
        "/Users/x/Application Support/lithemux-hook",
    )))
    .unwrap();
    assert_eq!(
        settings["hooks"]["Stop"][0]["hooks"][0]["command"],
        "'/Users/x/Application Support/lithemux-hook' stop"
    );
}
