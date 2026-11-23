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

// PLAN §2.2: categories and sessions are created from the sidebar, where
// the user types a name, not a slug.
#[test]
fn a_typed_name_gives_a_valid_slug() {
    use crate::modules::category::domain::slug_policy::slug_from_name;
    assert_eq!(slug_from_name("My API").as_deref(), Some("my-api"));
    assert_eq!(
        slug_from_name("  Diana — Calidad 2!  ").as_deref(),
        Some("diana-calidad-2")
    );
    assert_eq!(
        slug_from_name("already-a-slug").as_deref(),
        Some("already-a-slug")
    );
    assert_eq!(slug_from_name("¡¡¡"), None);
    assert_eq!(slug_from_name(""), None);
    let long = slug_from_name(&"word ".repeat(30)).unwrap();
    assert!(crate::modules::category::domain::slug_policy::is_valid_slug(&long));
}

// PLAN §2.3: each new terminal needs a name of its own.
#[test]
fn a_taken_slug_gets_the_next_free_number() {
    use crate::modules::category::domain::slug_policy::free_slug;
    assert_eq!(free_slug("terminal", |_| false), "terminal");
    let taken = ["terminal", "terminal-2"];
    assert_eq!(
        free_slug("terminal", |slug| taken.contains(&slug)),
        "terminal-3"
    );
}

// docs/decisiones.md D-07 + ronda 3B bug 1: the number that tells two
// slugs apart never takes a slug past 48 characters
#[test]
fn a_numbered_slug_stays_within_forty_eight_characters() {
    use crate::modules::category::domain::slug_policy::free_slug;
    let base = "a".repeat(48);
    let second_too = format!("{}-2", "a".repeat(46));
    let taken = |slug: &str| slug == base || slug == second_too;

    let second = free_slug(&base, |slug| slug == base);
    let third = free_slug(&base, taken);

    assert_eq!(second, format!("{}-2", "a".repeat(46)));
    assert_eq!(third, format!("{}-3", "a".repeat(46)));
    assert!(is_valid_slug(&second) && is_valid_slug(&third));
}

// ronda 3B bug 1: cut where a hyphen was, no slug ends in «--2»
#[test]
fn a_cut_base_does_not_leave_a_hyphen_before_the_number() {
    use crate::modules::category::domain::slug_policy::free_slug;
    let base = format!("{}-b", "a".repeat(45));

    let numbered = free_slug(&base, |slug| slug == base);

    assert!(is_valid_slug(&numbered), "{numbered}");
    assert!(!numbered.contains("--"), "{numbered}");
}

// ronda 3B bug 2: Spanish letters keep their letter, without the accent
#[test]
fn accented_letters_give_their_plain_letter_in_a_slug() {
    use crate::modules::category::domain::slug_policy::slug_from_name;
    assert_eq!(slug_from_name("Migración").as_deref(), Some("migracion"));
    assert_eq!(slug_from_name("Auditoría").as_deref(), Some("auditoria"));
    assert_eq!(
        slug_from_name("Ñandú Pingüino").as_deref(),
        Some("nandu-pinguino")
    );
    // The same, written as a letter followed by its accent (NFD, as macOS
    // gives file names).
    assert_eq!(
        slug_from_name("Migracio\u{301}n").as_deref(),
        Some("migracion")
    );
    assert_eq!(
        slug_from_name("N\u{303}andu\u{301}").as_deref(),
        Some("nandu")
    );
}
