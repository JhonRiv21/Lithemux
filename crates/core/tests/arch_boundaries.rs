#![allow(clippy::unwrap_used, clippy::expect_used)]

#[path = "arch/rules.rs"]
mod rules;

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
