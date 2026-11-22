use std::fs;
use std::path::PathBuf;

use lithemux_core::modules::category::get_category;
use lithemux_core::modules::launch::{
    ClaudeProjectsTranscriptLocator, LaunchEnvironment, LaunchPorts, ShellMode, TranscriptLocator,
    plan_launch,
};
use lithemux_core::modules::session::Kind;

use crate::support::fake_shell_probe::FakeShellProbe;
use crate::support::in_memory_launch_ports::{FixedHookBinary, HOOK_PATH, InMemoryLocks};
use crate::support::in_memory_store::{InMemoryBoards, InMemoryStore};
use crate::support::recording_ports::RecordingNotices;
use crate::support::sample_data::{add_category, add_session};

const ID: &str = "7d1c3f52-9a7e-4c1b-8f3e-2b9d5a6c2222";

fn write_transcript(home: &std::path::Path, project: &str, id: &str) {
    let dir = home.join(".claude/projects").join(project);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join(format!("{id}.jsonl")), "{}\n").unwrap();
}

// PLAN §0.3: ~/.claude/projects/*/<claude_session_id>.jsonl
#[test]
fn a_transcript_is_found_in_any_project_folder() {
    let home = tempfile::tempdir().unwrap();
    let locator = ClaudeProjectsTranscriptLocator::in_config_dir(home.path().join(".claude"));
    assert!(!locator.exists(ID), "no projects folder yet");

    write_transcript(
        home.path(),
        "-Users-x-other",
        "ffffffff-0000-4000-8000-000000000000",
    );
    assert!(!locator.exists(ID));

    write_transcript(home.path(), "-Users-x-work", ID);
    assert!(locator.exists(ID));
}

// PLAN principle 4: only a well-formed id is ever turned into a path
#[test]
fn a_malformed_id_never_matches() {
    let home = tempfile::tempdir().unwrap();
    write_transcript(home.path(), "p", "x");
    fs::write(home.path().join(".claude/secret.jsonl"), "{}").unwrap();
    let locator = ClaudeProjectsTranscriptLocator::in_config_dir(home.path().join(".claude"));
    assert!(!locator.exists("x"));
    assert!(!locator.exists("../secret"));
    assert!(!locator.exists(""));
}

// PLAN §0.7: --session-id vs --resume depending on the transcript, with a temporary HOME
#[test]
fn the_launcher_resumes_only_when_the_transcript_exists_on_disk() {
    let home = tempfile::tempdir().unwrap();
    let store = InMemoryStore::new();
    add_category(&store, &InMemoryBoards::default(), "demo");
    let session = add_session(&store, &store, "demo/ejecutora", Kind::Member);
    let claude_id = session.claude_session_id.clone().unwrap();
    let category = get_category(&store, "demo").unwrap();

    let probe = FakeShellProbe::new()
        .executable("/bin/claude")
        .executable(HOOK_PATH)
        .answers(
            ShellMode::LoginInteractive,
            "command -v claude",
            "/bin/claude",
        );
    let locator = ClaudeProjectsTranscriptLocator::in_config_dir(home.path().join(".claude"));
    let ports = LaunchPorts {
        categories: &store,
        sessions: &store,
        settings: &store,
        shell: &probe,
        notices: &RecordingNotices::default(),
        hook_binary: &FixedHookBinary,
        transcripts: &locator,
        locks: &InMemoryLocks::default(),
    };
    let environment = LaunchEnvironment {
        shell: PathBuf::from("/fake/zsh"),
        lithemux_home: None,
    };

    let fresh = plan_launch(&ports, &environment, &category, &session).unwrap();
    assert_eq!(
        fresh.args[..2],
        ["--session-id".to_string(), claude_id.clone()]
    );

    write_transcript(home.path(), "-work-demo", &claude_id);
    let resumed = plan_launch(&ports, &environment, &category, &session).unwrap();
    assert_eq!(resumed.args[..2], ["--resume".to_string(), claude_id]);
}
