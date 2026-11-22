use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use lithemux_core::modules::category::{FsBoardStore, SqliteCategoryRepository};
use lithemux_core::modules::launch::{
    ClaudeProjectsTranscriptLocator, FlockSessionLocks, LaunchEnvironment, LaunchPorts,
    LoginShellProbe, TerminalSessionRunner, hooks_settings_json, open_session,
};
use lithemux_core::modules::session::{
    Kind, SessionRef, SqliteSessionRepository, find_session_by_id,
};
use lithemux_core::modules::settings::{SqliteSettingsRepository, set_config};

use crate::support::in_memory_launch_ports::HookBinaryAt;
use crate::support::recording_ports::RecordingNotices;
use crate::support::sample_data::{add_category, add_session};
use crate::temp_home::TempHome;

fn fake_claude() -> String {
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fakes/fake-claude").to_string()
}

fn fake_shell() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fakes/fake-shell")
}

// PLAN §0.3 + docs/decisiones.md: the whole launch against a fake claude
// fixed with claude_path. No test runs the real claude.
#[test]
fn open_runs_the_fixed_claude_with_the_planned_arguments_and_environment() {
    let temp = TempHome::new();
    let db = temp.db();
    let (categories, sessions, settings) = (
        SqliteCategoryRepository::new(&db),
        SqliteSessionRepository::new(&db),
        SqliteSettingsRepository::new(&db),
    );
    add_category(&categories, &FsBoardStore::new(&temp.home), "demo");
    let session = add_session(&categories, &sessions, "demo/ejecutora", Kind::Child);
    let workdir = temp.dir.path().join("work");
    fs::create_dir_all(&workdir).unwrap();
    sessions_set_cwd(&db, &session.id, &workdir);
    set_config(&settings, "claude_path", &fake_claude()).unwrap();
    fs::write(temp.home.path().join("fake-claude.exit"), "7").unwrap();

    // Any executable file will do: this test never fires a hook.
    let hook = temp.dir.path().join("lithemux-hook");
    fs::write(&hook, "#!/usr/bin/true\n").unwrap();
    fs::set_permissions(&hook, fs::Permissions::from_mode(0o755)).unwrap();

    let notices = RecordingNotices::default();
    let ports = LaunchPorts {
        categories: &categories,
        sessions: &sessions,
        settings: &settings,
        shell: &LoginShellProbe::new(fake_shell()),
        notices: &notices,
        hook_binary: &HookBinaryAt(hook.clone()),
        transcripts: &ClaudeProjectsTranscriptLocator::in_config_dir(temp.dir.path().join("cfg")),
        locks: &FlockSessionLocks::new(&temp.home),
        runner: &TerminalSessionRunner,
    };
    let environment = LaunchEnvironment {
        shell: PathBuf::from(fake_shell()),
        lithemux_home: Some(temp.home.path().to_path_buf()),
    };

    let code = open_session(
        &ports,
        &environment,
        &SessionRef::parse("demo/ejecutora").unwrap(),
    );

    assert_eq!(code.unwrap(), 7, "the exit code of the child is returned");
    let log = fs::read_to_string(temp.home.path().join("fake-claude.log")).unwrap();
    let claude_id = session.claude_session_id.clone().unwrap();
    let expected = [
        format!("cwd={}", workdir.canonicalize().unwrap().display()),
        format!("session_env={}", session.id),
        format!("home_env={}", temp.home.path().display()),
        "arg=--session-id".to_string(),
        format!("arg={claude_id}"),
        "arg=-n".to_string(),
        "arg=demo-ejecutora".to_string(),
        "arg=--settings".to_string(),
        format!("arg={}", hooks_settings_json(&hook)),
    ];
    assert_eq!(log.lines().collect::<Vec<_>>(), expected);
    assert!(notices.warnings.borrow().is_empty());

    let (_, stored) = find_session_by_id(&categories, &sessions, &session.id)
        .unwrap()
        .unwrap();
    assert_eq!(stored.claude_version.as_deref(), Some("9.9.9"));
}

fn sessions_set_cwd(
    db: &lithemux_core::platform::store::connection::Db,
    id: &str,
    cwd: &std::path::Path,
) {
    db.conn()
        .execute(
            "UPDATE sessions SET cwd = ?2 WHERE id = ?1",
            rusqlite::params![id, cwd.canonicalize().unwrap().display().to_string()],
        )
        .unwrap();
}
