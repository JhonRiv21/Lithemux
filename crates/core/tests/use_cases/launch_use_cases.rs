use std::path::PathBuf;

use lithemux_core::modules::category::get_category;
use lithemux_core::modules::launch::{
    LaunchError, SessionLocks, ShellMode, hooks_settings_json, open_session, plan_launch,
};
use lithemux_core::modules::session::{
    Kind, Profile, SessionRef, create_session, find_session_by_id, record_claude_version,
};

use crate::support::fake_shell_probe::FakeShellProbe;
use crate::support::in_memory_launch_ports::{HOOK_PATH, RecordingRunner};
use crate::support::sample_data::{NOW, new_session};

use crate::launch_world::{CLAUDE, World, environment, target};

// PLAN §0.3: new or resume, decided by the transcript
#[test]
fn the_first_launch_creates_the_conversation_and_later_ones_resume_it() {
    let world = World::new();
    let category = get_category(&world.store, "demo").unwrap();
    let claude_id = world.executor.claude_session_id.clone().unwrap();

    let first = plan_launch(&world.ports(), &environment(), &category, &world.executor).unwrap();
    assert_eq!(
        first.args[..2],
        ["--session-id".to_string(), claude_id.clone()]
    );

    world
        .transcripts
        .existing
        .borrow_mut()
        .push(claude_id.clone());
    let second = plan_launch(&world.ports(), &environment(), &category, &world.executor).unwrap();
    assert_eq!(second.args[..2], ["--resume".to_string(), claude_id]);
}

// PLAN §0.3: the resolved claude, the name and the hooks
#[test]
fn the_plan_runs_the_resolved_claude_with_name_and_hooks() {
    let world = World::new();
    let category = get_category(&world.store, "demo").unwrap();
    let plan = plan_launch(&world.ports(), &environment(), &category, &world.executor).unwrap();
    assert_eq!(plan.program, PathBuf::from(CLAUDE));
    assert_eq!(
        plan.args[2..4],
        ["-n".to_string(), "demo-ejecutora".to_string()]
    );
    assert_eq!(plan.args[4], "--settings");
    assert_eq!(plan.args[5], hooks_settings_json(HOOK_PATH.as_ref()));
    assert_eq!(plan.cwd, PathBuf::from("/work/demo"));
}

// docs/decisiones.md: spawn + wait, exit code of the child
#[test]
fn open_returns_the_exit_code_of_the_session() {
    let world = World::new();
    let runner = RecordingRunner::exiting_with(7);
    assert_eq!(
        open_session(&world.ports(), &runner, &environment(), &target()).unwrap(),
        7
    );
    assert_eq!(runner.plans.borrow().len(), 1);
}

// PLAN §2.0 bug 9: without the hook the session would run with no state and
// no context, so it does not start at all
#[test]
fn open_fails_before_launching_when_the_hook_binary_is_not_executable() {
    let mut world = World::new();
    world.probe = FakeShellProbe::new().executable(CLAUDE).answers(
        ShellMode::LoginInteractive,
        "command -v claude",
        CLAUDE,
    );
    let runner = RecordingRunner::exiting_with(0);

    let error = open_session(&world.ports(), &runner, &environment(), &target()).unwrap_err();

    assert!(matches!(error, LaunchError::HookNotExecutable(_)));
    assert!(error.to_string().contains(HOOK_PATH), "{error}");
    assert!(runner.plans.borrow().is_empty());
    assert_eq!(world.locks.holder(&world.executor.id), None);
}

// docs/decisiones.md: `open` stores the version claude reports
#[test]
fn open_records_the_claude_version() {
    let world = World::new();
    let runner = RecordingRunner::exiting_with(0);
    open_session(&world.ports(), &runner, &environment(), &target()).unwrap();
    let (_, session) = find_session_by_id(&world.store, &world.store, &world.executor.id)
        .unwrap()
        .unwrap();
    assert_eq!(session.claude_version.as_deref(), Some("2.1.330"));
}

// docs/decisiones.md: a failing `--version` never blocks the launch
#[test]
fn open_still_launches_when_the_version_is_unknown() {
    let mut world = World::new();
    world.probe = FakeShellProbe::new()
        .executable(CLAUDE)
        .executable(HOOK_PATH)
        .answers(ShellMode::LoginInteractive, "command -v claude", CLAUDE);
    let runner = RecordingRunner::exiting_with(0);
    assert_eq!(
        open_session(&world.ports(), &runner, &environment(), &target()).unwrap(),
        0
    );
    let (_, session) = find_session_by_id(&world.store, &world.store, &world.executor.id)
        .unwrap()
        .unwrap();
    assert_eq!(session.claude_version, None);
}

// PLAN §3.2, B5: a version that could not be read this time is unknown, not
// the one of the run before; otherwise the notice of a new version would
// stay after the session was resumed on it.
#[test]
fn a_launch_whose_version_cannot_be_read_forgets_the_version_of_the_run_before() {
    let mut world = World::new();
    record_claude_version(&world.store, &world.executor.id, "2.1.300").unwrap();
    world.probe = FakeShellProbe::new()
        .executable(CLAUDE)
        .executable(HOOK_PATH)
        .answers(ShellMode::LoginInteractive, "command -v claude", CLAUDE);
    let runner = RecordingRunner::exiting_with(0);
    open_session(&world.ports(), &runner, &environment(), &target()).unwrap();
    let (_, session) = find_session_by_id(&world.store, &world.store, &world.executor.id)
        .unwrap()
        .unwrap();
    assert_eq!(session.claude_version, None);
}

// PLAN §0.3 order, step 3: the fallback is announced before launching
#[test]
fn the_fallback_warning_reaches_the_user() {
    let mut world = World::new();
    world.probe = FakeShellProbe::new()
        .executable(CLAUDE)
        .executable(HOOK_PATH)
        .answers(ShellMode::Login, "command -v claude", CLAUDE);
    let runner = RecordingRunner::exiting_with(0);
    open_session(&world.ports(), &runner, &environment(), &target()).unwrap();
    assert_eq!(world.notices.warnings.borrow().len(), 1);
}

// PLAN §0.3: shell profile, no hooks; it needs neither claude nor a lock
#[test]
fn a_shell_session_runs_the_login_shell_without_claude() {
    let mut world = World::new();
    world.probe = FakeShellProbe::new();
    let mut new = new_session("demo/term", Kind::Member);
    new.profile = Profile::Shell;
    let (_, terminal) = create_session(&world.store, &world.store, new, NOW).unwrap();
    let runner = RecordingRunner::exiting_with(0);

    let code = open_session(
        &world.ports(),
        &runner,
        &environment(),
        &SessionRef::parse("demo/term").unwrap(),
    )
    .unwrap();

    assert_eq!(code, 0);
    let plans = runner.plans.borrow();
    assert_eq!(plans[0].program, PathBuf::from("/fake/zsh"));
    assert_eq!(plans[0].args, vec!["-l".to_string()]);
    assert!(world.probe.calls.borrow().is_empty());
    assert_eq!(world.locks.holder(&terminal.id), None);
}

// PLAN §0.6
#[test]
fn opening_a_session_that_does_not_exist_fails() {
    let world = World::new();
    let runner = RecordingRunner::exiting_with(0);
    let missing = SessionRef::parse("demo/nadie").unwrap();
    assert!(matches!(
        open_session(&world.ports(), &runner, &environment(), &missing),
        Err(LaunchError::Session(_))
    ));
}
