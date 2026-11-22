#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn lint(rules: Option<&Path>, files: &[&Path]) -> Output {
    let mut command = Command::new(workspace().join("scripts/lint-findings"));
    if let Some(rules) = rules {
        command.env("FINDINGS_DIR", rules);
    }
    command.args(files).output().unwrap()
}

/// A rules folder with one sample rule: "no dbg! left behind".
fn sample_rules(bad: &str, good: &str) -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let rule = dir.path().join("H-900");
    fs::create_dir_all(&rule).unwrap();
    fs::write(rule.join("pattern"), "dbg!\\(\n").unwrap();
    fs::write(rule.join("bad.txt"), bad).unwrap();
    fs::write(rule.join("good.txt"), good).unwrap();
    fs::write(rule.join("README.md"), "# H-900\n\nSample rule.\n").unwrap();
    dir
}

fn source(dir: &TempDir, name: &str, text: &str) -> PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, text).unwrap();
    path
}

// CLAUDE.md «Pruebas» 7: the repo is clean under its own rules
#[test]
fn the_repository_passes_its_own_findings_lint() {
    let output = lint(None, &[]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    assert!(stdout.contains("no findings"), "{stdout}");
    // "N rule(s), M file(s), no findings": it must have looked at the repo.
    let reviewed: usize = stdout
        .split(", ")
        .find_map(|part| part.strip_suffix(" file(s)"))
        .and_then(|count| count.trim().parse().ok())
        .unwrap();
    assert!(reviewed > 200, "only {reviewed} files were reviewed");
}

// Audit of the delta: with nothing to review (no git repository to list the
// files) the lint says so instead of reporting a clean repo
#[test]
fn reviewing_no_files_is_a_failure_and_not_a_clean_result() {
    let output = Command::new(workspace().join("scripts/lint-findings"))
        .env("GIT_DIR", "/nonexistent/lithemux-no-repository")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("no files to check"));
}

// CLAUDE.md «Pruebas» 7: a rule flags its bad example in real code
#[test]
fn a_rule_flags_the_code_it_was_written_for() {
    let rules = sample_rules("let x = dbg!(compute());\n", "let x = compute();\n");
    let file = source(&rules, "leftover.rs", "fn main() {\n    dbg!(1);\n}\n");

    let output = lint(Some(rules.path()), &[&file]);

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("H-900"), "{stdout}");
    assert!(stdout.contains("leftover.rs:2:"), "{stdout}");
}

// CLAUDE.md «Pruebas» 7: and lets the good example through
#[test]
fn a_rule_lets_clean_code_through() {
    let rules = sample_rules("let x = dbg!(compute());\n", "let x = compute();\n");
    let file = source(
        &rules,
        "clean.rs",
        "fn main() {\n    println!(\"ok\");\n}\n",
    );
    let output = lint(Some(rules.path()), &[&file]);
    assert_eq!(output.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&output.stdout).contains("1 rule(s)"));
}

// CLAUDE.md «Pruebas» 7: every rule is proven against its two examples first
#[test]
fn a_rule_that_misses_its_bad_example_stops_the_lint() {
    let rules = sample_rules("let x = compute();\n", "let y = 1;\n");
    let output = lint(Some(rules.path()), &[]);
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("does not flag its own bad example"));
}

#[test]
fn a_rule_that_flags_its_good_example_stops_the_lint() {
    let rules = sample_rules("dbg!(1);\n", "dbg!(2);\n");
    let output = lint(Some(rules.path()), &[]);
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("flags its own good example"));
}

#[test]
fn a_rule_without_its_examples_or_its_reason_stops_the_lint() {
    let rules = sample_rules("dbg!(1);\n", "ok\n");
    fs::remove_file(rules.path().join("H-900/README.md")).unwrap();
    let output = lint(Some(rules.path()), &[]);
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("missing README.md"));
}

// A rule applies to Rust sources unless it says otherwise.
#[test]
fn a_rule_only_looks_at_the_paths_it_declares() {
    let rules = sample_rules("dbg!(1);\n", "ok\n");
    let notes = source(&rules, "notes.md", "remember to remove dbg!(x)\n");
    assert_eq!(lint(Some(rules.path()), &[&notes]).status.code(), Some(0));
    fs::write(rules.path().join("H-900/paths"), "\\.md$\n").unwrap();
    assert_eq!(lint(Some(rules.path()), &[&notes]).status.code(), Some(1));
}

// PLAN §2.0 point 19: a rule can name the one place where the pattern belongs.
#[test]
fn a_rule_leaves_alone_the_paths_it_excludes() {
    let rules = sample_rules("dbg!(1);\n", "ok\n");
    let allowed = source(&rules, "debug_console.rs", "fn main() {\n    dbg!(1);\n}\n");
    let elsewhere = source(&rules, "leftover.rs", "fn main() {\n    dbg!(1);\n}\n");
    fs::write(rules.path().join("H-900/exclude"), "debug_console\\.rs$\n").unwrap();

    assert_eq!(lint(Some(rules.path()), &[&allowed]).status.code(), Some(0));
    assert_eq!(
        lint(Some(rules.path()), &[&elsewhere]).status.code(),
        Some(1)
    );
}
