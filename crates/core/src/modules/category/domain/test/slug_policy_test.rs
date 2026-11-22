use crate::modules::category::domain::slug::Slug;
use crate::modules::category::domain::slug_policy::is_valid_slug;

// PLAN §0.2
#[test]
fn slug_accepts_lowercase_letters_digits_and_hyphens() {
    for slug in ["selektal", "planificadora", "turnos-positiva", "v2", "a"] {
        assert!(is_valid_slug(slug), "{slug} should be valid");
    }
}

// PLAN §0.2
#[test]
fn slug_rejects_anything_outside_the_allowed_charset() {
    for slug in [
        "",
        "Selektal",
        "con espacio",
        "ñandú",
        "a_b",
        "a/b",
        "a.b",
        "tab\t",
    ] {
        assert!(!is_valid_slug(slug), "{slug:?} should be invalid");
    }
}

// docs/decisiones.md: slugs without a leading or trailing hyphen
#[test]
fn slug_rejects_a_leading_or_trailing_hyphen() {
    assert!(!is_valid_slug("-foo"));
    assert!(!is_valid_slug("foo-"));
    assert!(!is_valid_slug("-"));
}

// PLAN §0.2
#[test]
fn slug_newtype_only_holds_valid_slugs() {
    assert_eq!(Slug::parse("demo").unwrap().as_str(), "demo");
    assert!(Slug::parse("Demo").is_err());
}

// docs/decisiones.md D-07: at most 48 characters
#[test]
fn slug_is_capped_at_forty_eight_characters() {
    assert!(is_valid_slug(&"a".repeat(48)));
    assert!(!is_valid_slug(&"a".repeat(49)));
}
