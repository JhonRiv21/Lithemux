//! PLAN §3.7, carried over from 3B: a `claude_path` that cannot run says so,
//! and why, on the card and in app.log, and no other `claude` runs in its
//! place. With the real probe: what is on disk is really on disk.

use std::fs;
use std::os::unix::fs::PermissionsExt;

use lithemux_app::session_host::{Failure, Phase, SessionHost};
use lithemux_core::modules::launch::ClaudePathProblem;
use lithemux_core::modules::session::Profile;
use lithemux_core::modules::settings::{SettingKey, SqliteSettingsRepository, write_setting};

use crate::hosted_world::{HostedWorld, core_fake, open};

/// A login shell whose PATH has a working fake `claude` (fakes/on-path):
/// the one the app would run if it fell back to the PATH.
const SHELL_THAT_FINDS_A_CLAUDE: &str = "fake-shell-interactive";

fn host_with_claude_path(world: &HostedWorld, value: &str) -> SessionHost {
    write_setting(
        &SqliteSettingsRepository::new(&world.db()),
        SettingKey::ClaudePath,
        value,
    )
    .unwrap();
    let mut config = world.config();
    config.shell = core_fake(SHELL_THAT_FINDS_A_CLAUDE);
    SessionHost::start(config).unwrap().0
}

fn assert_refused(world: &HostedWorld, value: &str, expected: ClaudePathProblem) {
    let session = world.add_session("api", Profile::Claude);
    let mut host = host_with_claude_path(world, value);

    let phase = open(&mut host, "api");

    assert!(
        matches!(
            &phase,
            Phase::Failed(Failure::ClaudePathUnusable { value: said, problem })
                if said == value && *problem == expected
        ),
        "{phase:?}"
    );
    assert!(world.fake_claude_runs().is_empty(), "another claude ran");
    let log = fs::read_to_string(world.home.app_log_path()).unwrap_or_default();
    let line = log.lines().find(|line| line.contains(&session.id));
    assert!(
        line.is_some_and(
            |line| line.contains("claude_path") && line.contains(&expected.to_string())
        ),
        "{log}"
    );
}

#[test]
fn a_claude_path_where_nothing_is_fails_the_session_and_runs_nothing() {
    assert_refused(
        &HostedWorld::new(),
        "/gone/nowhere",
        ClaudePathProblem::Missing,
    );
}

#[test]
fn a_claude_path_with_json_quotes_fails_the_session_and_runs_nothing() {
    let fake = core_fake("fake-claude").display().to_string();
    assert_refused(
        &HostedWorld::new(),
        &format!("\"{fake}\""),
        ClaudePathProblem::Quoted,
    );
}

// ronda 3C: a real folder.
#[test]
fn a_claude_path_to_a_folder_fails_the_session_and_runs_nothing() {
    let world = HostedWorld::new();
    let folder = world.dir.path().join("claude-folder");
    fs::create_dir(&folder).unwrap();

    assert_refused(
        &world,
        &folder.display().to_string(),
        ClaudePathProblem::Directory,
    );
}

// ronda 3C: a real file, without the right to run: a copy of the fake
// `claude`, so it would work if it could run.
#[test]
fn a_claude_path_to_a_file_that_cannot_run_fails_the_session_and_runs_nothing() {
    let world = HostedWorld::new();
    let copy = world.dir.path().join("fake-claude-copy");
    fs::copy(core_fake("fake-claude"), &copy).unwrap();
    fs::set_permissions(&copy, fs::Permissions::from_mode(0o644)).unwrap();

    assert_refused(
        &world,
        &copy.display().to_string(),
        ClaudePathProblem::NotExecutable,
    );
}
