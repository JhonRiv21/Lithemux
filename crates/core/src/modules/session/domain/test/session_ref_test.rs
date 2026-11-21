use crate::modules::session::domain::session_ref::SessionRef;

// PLAN §0.6: sessions are addressed as <cat>/<slug>
#[test]
fn a_reference_is_category_slash_session() {
    let target = SessionRef::parse("selektal/ejecutora").unwrap();
    assert_eq!(target.category.as_str(), "selektal");
    assert_eq!(target.session.as_str(), "ejecutora");
    assert_eq!(target.to_string(), "selektal/ejecutora");
    assert_eq!(target.messaging_name(), "selektal-ejecutora");
}

// PLAN §0.6
#[test]
fn a_reference_needs_both_parts() {
    for value in ["selektal", "selektal/", "/ejecutora", "a/b/c", ""] {
        assert!(SessionRef::parse(value).is_err(), "{value:?} should fail");
    }
}

// PLAN §0.2: both parts are slugs
#[test]
fn both_parts_of_a_reference_must_be_valid_slugs() {
    assert!(SessionRef::parse("Selektal/ejecutora").is_err());
    assert!(SessionRef::parse("selektal/la ejecutora").is_err());
}
