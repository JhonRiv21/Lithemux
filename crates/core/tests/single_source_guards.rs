#![allow(clippy::unwrap_used, clippy::expect_used)]

#[path = "guards/real_programs.rs"]
mod real_programs;

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn rust_sources(dir: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_sources(&path, files);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
}

/// Text that only code reading Claude Code's own files would contain.
const CLAUDE_INTERNALS: [&str; 7] = [
    "\".claude\"",
    ".claude/",
    ".jsonl",
    "CLAUDE_CONFIG_DIR",
    "sessions/",
    "jobs/",
    "\"projects\"",
];
const ONLY_ALLOWED_IN: &str = "modules/launch/infrastructure/transcript_locator.rs";

fn internals_in(text: &str) -> Vec<&'static str> {
    CLAUDE_INTERNALS
        .into_iter()
        .filter(|needle| {
            text.lines()
                .any(|line| !line.trim_start().starts_with("//") && line.contains(needle))
        })
        .collect()
}

// The guard itself: it must see what it is supposed to forbid.
#[test]
fn the_internals_guard_recognizes_a_read_of_claude_files() {
    let bad =
        "let dir = home.join(\".claude\").join(\"projects\");\nlet f = format!(\"{id}.jsonl\");";
    assert_eq!(
        internals_in(bad),
        vec!["\".claude\"", ".jsonl", "\"projects\""]
    );
    assert_eq!(
        internals_in("fs::read(\"/Users/x/.claude/sessions/1.json\")"),
        vec![".claude/", "sessions/"]
    );
    let good = "// lives in ~/.claude/projects, but this is a comment\nlet id = session.claude_session_id.clone();";
    assert!(internals_in(good).is_empty());
}

// PLAN principle 4: the transcript's existence is the only thing lithemux
// looks at inside Claude Code's files, and it does so in exactly one place.
#[test]
fn claude_internal_paths_appear_only_in_the_transcript_locator() {
    let root = workspace().join("crates");
    let mut offenders = Vec::new();
    let mut allowed_file_seen = false;
    for crate_dir in fs::read_dir(&root).unwrap().flatten() {
        let src = crate_dir.path().join("src");
        if !src.is_dir() {
            continue;
        }
        let mut files = Vec::new();
        rust_sources(&src, &mut files);
        for file in files {
            let relative = file
                .strip_prefix(&root)
                .unwrap()
                .to_string_lossy()
                .to_string();
            let found = internals_in(&fs::read_to_string(&file).unwrap());
            if relative.ends_with(ONLY_ALLOWED_IN) {
                allowed_file_seen = !found.is_empty();
            } else if relative.contains("/test/") {
                continue;
            } else if !found.is_empty() {
                offenders.push(format!("{relative}: {found:?}"));
            }
        }
    }
    assert!(
        allowed_file_seen,
        "{ONLY_ALLOWED_IN} should be the one place that knows"
    );
    assert!(
        offenders.is_empty(),
        "Claude Code internals outside the locator:\n{}",
        offenders.join("\n")
    );
}

const TELEMETRY_PREFIXES: [&str; 11] = [
    "sentry",
    "posthog",
    "opentelemetry",
    "segment",
    "rudder",
    "mixpanel",
    "amplitude",
    "bugsnag",
    "datadog",
    "dogstatsd",
    "tracing-opentelemetry",
];
const HTTP_CLIENTS: [&str; 9] = [
    "reqwest",
    "hyper",
    "hyper-util",
    "ureq",
    "isahc",
    "surf",
    "curl",
    "attohttpc",
    "h2",
];

fn forbidden_packages(lock: &str) -> Vec<String> {
    lock.lines()
        .filter_map(|line| line.strip_prefix("name = \""))
        .map(|rest| rest.trim_end_matches('"'))
        .filter(|name| {
            TELEMETRY_PREFIXES
                .iter()
                .any(|prefix| name.starts_with(prefix))
                || HTTP_CLIENTS.contains(name)
        })
        .map(str::to_string)
        .collect()
}

// The guard itself.
#[test]
fn the_dependency_guard_recognizes_telemetry_and_http_clients() {
    let lock = "[[package]]\nname = \"serde\"\n\n[[package]]\nname = \"sentry-core\"\n\n[[package]]\nname = \"reqwest\"\n\n[[package]]\nname = \"posthog-rs\"\n\n[[package]]\nname = \"hyperfine-like\"\n";
    assert_eq!(
        forbidden_packages(lock),
        vec!["sentry-core", "reqwest", "posthog-rs"]
    );
}

// PLAN principle 2: zero telemetry and zero network in the whole dependency tree.
#[test]
fn cargo_lock_has_no_telemetry_sdk_and_no_http_client() {
    let lock = fs::read_to_string(workspace().join("Cargo.lock")).unwrap();
    assert!(
        lock.contains("name = \"rusqlite\""),
        "this is the real Cargo.lock"
    );
    let forbidden = forbidden_packages(&lock);
    assert!(
        forbidden.is_empty(),
        "forbidden packages in Cargo.lock: {forbidden:?}"
    );
}

fn real_programs_fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/guard-fixtures/real-programs")
        .join(name)
}

fn fixture_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap()
        .flatten()
        .map(|entry| entry.path())
        .collect();
    files.sort();
    files
}

// The guard itself: the good fixture passes.
#[test]
fn the_real_programs_guard_accepts_tests_that_only_run_fakes() {
    let dir = real_programs_fixture("good");
    let violations = real_programs::check_files(&dir, &fixture_files(&dir));
    assert!(violations.is_empty(), "{violations:?}");
}

// The guard itself: every bad file trips exactly the rule it is named after.
#[test]
fn the_real_programs_guard_flags_real_shells_and_the_real_claude() {
    let dir = real_programs_fixture("bad");
    let files = fixture_files(&dir);
    let violations = real_programs::check_files(&dir, &files);

    let flagged: BTreeSet<&str> = violations.iter().map(|v| v.file.as_str()).collect();
    assert_eq!(
        flagged.len(),
        files.len(),
        "every bad file is flagged: {violations:?}"
    );
    for violation in &violations {
        let expected = violation.rule.replace('-', "_");
        assert!(
            violation.file.starts_with(&expected),
            "{} tripped {}",
            violation.file,
            violation.rule
        );
    }
    let rules: BTreeSet<&str> = violations.iter().map(|v| v.rule).collect();
    assert_eq!(
        rules,
        BTreeSet::from(["real-claude", "real-shell", "shell-env"])
    );
}

// docs/decisiones.md D-13: no test runs a real shell or the real claude.
#[test]
fn no_test_runs_a_real_shell_or_the_real_claude() {
    let root = workspace().join("crates");
    let files = real_programs::test_sources(&root);
    assert!(
        files.len() > 40,
        "the test code was found ({} files)",
        files.len()
    );
    let violations = real_programs::check_files(&root, &files);
    assert!(
        violations.is_empty(),
        "tests must only run tests/fakes/fake-claude and fake-shell:\n{}",
        violations
            .iter()
            .map(|v| format!("  [{}] {}:{}", v.rule, v.file, v.line))
            .collect::<Vec<_>>()
            .join("\n")
    );
}
