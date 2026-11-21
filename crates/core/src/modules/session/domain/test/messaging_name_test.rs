use crate::modules::session::domain::messaging_name::{messaging_name, name_owner};

// PLAN §0.2
#[test]
fn messaging_name_is_category_slug_dash_session_slug() {
    assert_eq!(
        messaging_name("selektal", "planificadora"),
        "selektal-planificadora"
    );
}

// PLAN §0.2: the name must be unique across the whole system
#[test]
fn a_name_already_in_use_reports_its_owner() {
    let existing = [("selektal", "ejecutora"), ("demo", "planificadora")];
    assert_eq!(
        name_owner("demo-planificadora", existing),
        Some("demo/planificadora".to_string())
    );
}

// PLAN §0.2: different category/session pairs can produce the same name
#[test]
fn names_collide_across_categories_when_hyphens_line_up() {
    let existing = [("a-b", "c")];
    assert_eq!(
        name_owner(&messaging_name("a", "b-c"), existing),
        Some("a-b/c".to_string())
    );
}

// PLAN §0.2
#[test]
fn a_free_name_has_no_owner() {
    let existing = [("selektal", "ejecutora")];
    assert_eq!(name_owner("selektal-documental", existing), None);
}
