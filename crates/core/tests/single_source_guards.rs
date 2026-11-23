#![allow(clippy::unwrap_used, clippy::expect_used)]

#[path = "guards/process_spawning.rs"]
mod process_spawning;
#[path = "guards/real_programs.rs"]
mod real_programs;
// Shared with the architecture test; this binary does not use all of it.
#[allow(dead_code)]
#[path = "guards/tokens.rs"]
mod tokens;

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
const HTTP_CLIENTS: [&str; 16] = [
    "reqwest",
    "hyper",
    "hyper-util",
    "ureq",
    "isahc",
    "surf",
    "curl",
    "attohttpc",
    "h2",
    "tungstenite",
    "rustls",
    "native-tls",
    // Zed's own telemetry and network crates.
    "telemetry",
    "telemetry_events",
    "reqwest_client",
    "client",
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
    let lock = "[[package]]\nname = \"serde\"\n\n[[package]]\nname = \"sentry-core\"\n\n[[package]]\nname = \"reqwest\"\n\n[[package]]\nname = \"posthog-rs\"\n\n[[package]]\nname = \"hyperfine-like\"\n\n[[package]]\nname = \"http_client\"\n\n[[package]]\nname = \"telemetry_events\"\n\n[[package]]\nname = \"rustls\"\n\n[[package]]\nname = \"client\"\n";
    assert_eq!(
        forbidden_packages(lock),
        vec![
            "sentry-core",
            "reqwest",
            "posthog-rs",
            "telemetry_events",
            "rustls",
            "client"
        ]
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
    let mut files = Vec::new();
    rust_sources(dir, &mut files);
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
        BTreeSet::from([
            "inherited-env",
            "machine-env",
            "real-claude",
            "real-shell",
            "shell-by-name",
            "shell-env",
            "workspace-binary"
        ])
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

fn process_spawning_fixture(name: &str) -> Vec<(String, String)> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/guard-fixtures/process-spawning")
        .join(name);
    let mut files = Vec::new();
    every_file(&dir, &mut files);
    files.sort();
    files
        .into_iter()
        .map(|path| {
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            (name, fs::read_to_string(path).unwrap())
        })
        .collect()
}

/// Every file, whatever its extension: code can be pulled in from any.
fn every_file(dir: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            every_file(&path, files);
        } else {
            files.push(path);
        }
    }
}

// The guard itself: every way the audit found around the textual H-001.
#[test]
fn the_process_guard_flags_every_way_of_naming_a_process_builder() {
    let bad = process_spawning_fixture("bad");
    assert!(bad.len() >= 11, "{} bad examples", bad.len());
    for (file, source) in bad {
        let mentions = process_spawning::mentions_of_a_process_builder(&file, &source);
        assert!(!mentions.is_empty(), "{file} was not flagged");
    }
}

// The guard itself: similar names, strings and comments are not mentions.
#[test]
fn the_process_guard_accepts_code_that_goes_through_the_filter() {
    for (file, source) in process_spawning_fixture("good") {
        let mentions = process_spawning::mentions_of_a_process_builder(&file, &source);
        assert!(mentions.is_empty(), "{mentions:?}");
    }
}

// findings/H-001 + docs/decisiones.md D-22: only three files of production
// code may name a type that starts processes.
#[test]
fn production_code_names_a_process_builder_only_where_the_filter_is() {
    let crates = workspace().join("crates");
    let mut files = Vec::new();
    for entry in fs::read_dir(&crates).unwrap().flatten() {
        let src = entry.path().join("src");
        if src.is_dir() {
            every_file(&src, &mut files);
        }
    }
    assert!(files.len() > 150, "found {} files", files.len());
    assert!(
        files
            .iter()
            .any(|path| path.extension().is_none_or(|ext| ext != "rs")),
        "files that are not Rust are scanned too"
    );
    let mut mentions = Vec::new();
    let mut allowed_seen = 0;
    for path in files {
        let file = path
            .strip_prefix(&crates)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        if process_spawning::ALLOWED_FILES.contains(&file.as_str()) {
            allowed_seen += 1;
            continue;
        }
        let Ok(source) = fs::read_to_string(&path) else {
            continue;
        };
        mentions.extend(process_spawning::mentions_of_a_process_builder(
            &file, &source,
        ));
    }
    assert_eq!(allowed_seen, process_spawning::ALLOWED_FILES.len());
    assert!(
        mentions.is_empty(),
        "processes may only be built where the environment filter is applied:\n{}",
        mentions
            .iter()
            .map(|m| format!("  {}:{} names `{}`", m.file, m.line, m.name))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// The lines of one `[table]` of a Cargo manifest, without comments.
fn manifest_table(manifest: &str, table: &str) -> Vec<String> {
    manifest
        .lines()
        .skip_while(|line| line.trim() != format!("[{table}]"))
        .skip(1)
        .take_while(|line| !line.trim_start().starts_with('['))
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_string)
        .collect()
}

// docs/decisiones.md D-27 and D-64: cargo cannot inherit the workspace
// lints and override one, so `core` and `webview-spike` repeat them. This keeps the copy honest: the
// same lints as the workspace, and `unsafe_code` the only difference.
#[test]
fn core_has_the_lints_of_the_workspace_except_for_unsafe_code() {
    let workspace_manifest = fs::read_to_string(workspace().join("Cargo.toml")).unwrap();
    let core_manifest = fs::read_to_string(workspace().join("crates/core/Cargo.toml")).unwrap();

    assert_eq!(
        manifest_table(&core_manifest, "lints.clippy"),
        manifest_table(&workspace_manifest, "workspace.lints.clippy")
    );
    assert_eq!(
        manifest_table(&workspace_manifest, "workspace.lints.rust"),
        ["unsafe_code = \"forbid\""]
    );
    assert_eq!(
        manifest_table(&core_manifest, "lints.rust"),
        ["unsafe_code = \"deny\""]
    );
    // D-64: the browser spike's crate has the same exception, for its one
    // file of WebKit configuration.
    let spike_manifest =
        fs::read_to_string(workspace().join("crates/webview-spike/Cargo.toml")).unwrap();
    assert_eq!(
        manifest_table(&spike_manifest, "lints.clippy"),
        manifest_table(&workspace_manifest, "workspace.lints.clippy")
    );
    assert_eq!(
        manifest_table(&spike_manifest, "lints.rust"),
        ["unsafe_code = \"deny\""]
    );
    for other in ["cli", "hook", "app"] {
        let manifest =
            fs::read_to_string(workspace().join(format!("crates/{other}/Cargo.toml"))).unwrap();
        assert_eq!(
            manifest_table(&manifest, "lints"),
            ["workspace = true"],
            "{other}"
        );
    }
}
