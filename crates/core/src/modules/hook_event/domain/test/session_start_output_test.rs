use serde_json::Value;

use crate::modules::hook_event::domain::session_start_output::session_start_output;

// PLAN «Hechos verificados»: hookSpecificOutput.additionalContext reaches Claude
#[test]
fn the_context_travels_as_additional_context_of_session_start() {
    let output: Value = serde_json::from_str(&session_start_output("line 1\nline \"2\"")).unwrap();
    assert_eq!(
        output["hookSpecificOutput"]["hookEventName"],
        "SessionStart"
    );
    assert_eq!(
        output["hookSpecificOutput"]["additionalContext"],
        "line 1\nline \"2\""
    );
}
