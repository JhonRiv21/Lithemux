#![allow(clippy::unwrap_used, clippy::expect_used)]

#[path = "arch/foreign_domain_types.rs"]
mod foreign_domain_types;
#[path = "arch/rules.rs"]
mod rules;
// Shared with the other guards; this binary does not use all of it.
#[allow(dead_code)]
#[path = "guards/tokens.rs"]
mod tokens;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use rules::{Violation, check};

const RULES: [&str; 6] = [
    "domain-purity",
    "module-internals",
    "platform-independence",
    "test-isolation",
    "no-barrels",
    "no-junk-drawers",
];

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/arch-fixtures")
        .join(name)
}

fn describe(violations: &[Violation]) -> String {
    violations
        .iter()
        .map(|v| format!("  [{}] {}:{} {}", v.rule, v.file, v.line, v.detail))
        .collect::<Vec<_>>()
        .join("\n")
}

// CLAUDE.md «Pruebas» 5: the good fixture passes
#[test]
fn the_good_fixture_breaks_no_rule() {
    let violations = check(&fixture("good"));
    assert!(
        violations.is_empty(),
        "unexpected:\n{}",
        describe(&violations)
    );
}

// CLAUDE.md «Pruebas» 5: the bad fixture fails naming every rule
#[test]
fn the_bad_fixture_trips_every_rule() {
    let violations = check(&fixture("bad"));
    let tripped: BTreeSet<&str> = violations.iter().map(|v| v.rule).collect();
    for rule in RULES {
        assert!(
            tripped.contains(rule),
            "rule `{rule}` did not fire:\n{}",
            describe(&violations)
        );
    }
}

// Each bad file is named after the one rule it breaks, so a rule that fires
// on the wrong file (or not at all) is caught.
#[test]
fn every_bad_file_trips_exactly_the_rule_it_is_named_after() {
    let violations = check(&fixture("bad"));
    let mut files: BTreeSet<&str> = BTreeSet::new();
    for violation in &violations {
        files.insert(&violation.file);
        let expected = violation
            .file
            .rsplit('/')
            .find_map(|part| {
                RULES
                    .iter()
                    .find(|rule| part.starts_with(&rule.replace('-', "_")))
            })
            .unwrap_or_else(|| panic!("{} is not named after a rule", violation.file));
        assert_eq!(violation.rule, *expected, "{}", describe(&violations));
    }
    assert!(
        files.len() >= 10,
        "every bad file must be reported:\n{}",
        describe(&violations)
    );
}

// CLAUDE.md «Arquitectura» 2: the real code
#[test]
fn the_codebase_respects_the_boundaries() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let violations = check(&crates.canonicalize().unwrap());
    assert!(
        violations.is_empty(),
        "boundary violations:\n{}",
        describe(&violations)
    );
}

// docs/decisiones.md D-19: only `crates/app/src/<feature>/mod.rs` is a door.
#[test]
fn a_feature_door_exists_only_in_the_app_crate() {
    let violations = check(&fixture("bad"));
    for lookalike in [
        "tauri-app/src/no_barrels_other_crate/mod.rs",
        "core/src/app/src/no_barrels_nested_app/mod.rs",
        "app/src/terminal_view/no_barrels_inner/mod.rs",
    ] {
        assert!(
            violations
                .iter()
                .any(|violation| violation.rule == "no-barrels" && violation.file == lookalike),
            "{lookalike} was taken for a door:\n{}",
            describe(&violations)
        );
    }
}

// PLAN §2.0 point 12 (audit I2): a door also re-exports use cases, and a
// domain may only take the types a door exposes from its own domain,
// however the path to them is written
#[test]
fn a_domain_cannot_take_a_use_case_from_any_door_by_any_spelling() {
    let violations = check(&fixture("bad"));
    for (file, line) in [
        ("session/domain/domain_purity_foreign_use_case.rs", 1),
        (
            "session/domain/domain_purity_foreign_use_case_grouped.rs",
            3,
        ),
        (
            "session/domain/domain_purity_foreign_use_case_through_the_module.rs",
            4,
        ),
        (
            "session/domain/domain_purity_foreign_use_case_under_an_alias.rs",
            4,
        ),
        (
            "session/domain/domain_purity_foreign_use_case_through_super.rs",
            2,
        ),
        (
            "session/domain/domain_purity_foreign_use_case_inside_a_macro.rs",
            4,
        ),
        ("category/domain/domain_purity_own_use_case.rs", 1),
        (
            "session/domain/domain_purity_foreign_use_case_as_a_raw_identifier.rs",
            1,
        ),
    ] {
        let found: Vec<&Violation> = violations
            .iter()
            .filter(|violation| violation.file.ends_with(file))
            .collect();
        assert_eq!(found.len(), 1, "{file}:\n{}", describe(&violations));
        assert_eq!(found[0].rule, "domain-purity");
        assert_eq!(found[0].line, line, "{file}");
        assert!(found[0].detail.contains("create_category"), "{file}");
    }
}

// Audit I2: a use case cannot reach the door dressed as a domain name
#[test]
fn a_domain_cannot_reexport_what_is_not_its_own() {
    let violations = check(&fixture("bad"));
    let found: Vec<&Violation> = violations
        .iter()
        .filter(|violation| {
            violation
                .file
                .ends_with("domain_purity_reexported_use_case.rs")
        })
        .collect();
    assert!(
        found.iter().any(|v| v.detail.contains("re-exports")),
        "{}",
        describe(&violations)
    );
    assert!(found.iter().all(|v| v.rule == "domain-purity"));
}

// Audit of the delta: a glob over every module brings every door in at once
#[test]
fn a_domain_cannot_import_every_module_at_once() {
    let violations = check(&fixture("bad"));
    assert!(
        violations.iter().any(|violation| {
            violation
                .file
                .ends_with("domain_purity_every_module_at_once.rs")
                && violation.rule == "domain-purity"
                && violation.line == 1
        }),
        "{}",
        describe(&violations)
    );
}

// PLAN §2.1: the session host has no dependency on gpui, so it can be
// tested without a window and driven by any interface
#[test]
fn the_session_host_does_not_know_gpui() {
    let host = Path::new(env!("CARGO_MANIFEST_DIR")).join("../app/src/session_host");
    let mut files = Vec::new();
    rules::collect(&host, &mut files);
    assert!(files.len() >= 8, "found {} files of the host", files.len());
    for path in files {
        let source = std::fs::read_to_string(&path).unwrap();
        let mentions: Vec<usize> = tokens::tokens_of(&source)
            .unwrap()
            .iter()
            .filter(|token| matches!(token.ident(), Some("gpui" | "gpui_platform")))
            .map(|token| token.line)
            .collect();
        assert!(mentions.is_empty(), "{}:{mentions:?}", path.display());
    }
}

// The rule above reads tokens, so a mention in a comment is not a use
#[test]
fn a_use_of_gpui_is_told_from_a_mention_in_a_comment() {
    let uses = tokens::tokens_of("// no gpui here\nuse gpui::App;\n").unwrap();
    let lines: Vec<usize> = uses
        .iter()
        .filter(|token| token.ident() == Some("gpui"))
        .map(|token| token.line)
        .collect();
    assert_eq!(lines, [2]);
}
