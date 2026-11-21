use crate::modules::context::domain::role_text::role_text;

// PLAN §0.5: the context is short; a role is free text
#[test]
fn a_role_becomes_one_bounded_line_without_final_period() {
    assert_eq!(role_text("Planifica.\n  Revisa. "), "Planifica. Revisa");
    let long = "x".repeat(500);
    assert_eq!(role_text(&long).chars().count(), 160);
}
