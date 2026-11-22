use std::cell::Cell;
use std::path::PathBuf;

use lithemux_core::modules::category::get_category;
use lithemux_core::modules::launch::{
    LaunchEnvironment, LaunchError, LaunchPorts, SessionLocks, ShellMode, TranscriptLocator,
    hooks_settings_json, open_session, plan_launch,
};
use lithemux_core::modules::session::{
    Kind, Profile, Session, SessionRef, create_session, find_session_by_id,
};

use crate::support::fake_shell_probe::FakeShellProbe;
use crate::support::in_memory_launch_ports::{
    FixedHookBinary, HOOK_PATH, InMemoryLocks, InMemoryTranscripts, RecordingRunner,
};
use crate::support::in_memory_store::{InMemoryBoards, InMemoryStore};
use crate::support::recording_ports::RecordingNotices;
use crate::support::sample_data::{NOW, add_category, add_session, new_session};

const CLAUDE: &str = "/Users/x/.local/bin/claude";

struct World {
    store: InMemoryStore,
    probe: FakeShellProbe,
    notices: RecordingNotices,
    transcripts: InMemoryTranscripts,
    locks: InMemoryLocks,
    executor: Session,
}

impl World {
    fn new() -> Self {
        let store = InMemoryStore::new();
        add_category(&store, &InMemoryBoards::default(), "demo");
        let executor = add_session(&store, &store, "demo/ejecutora", Kind::Child);
        World {
            store,
            probe: FakeShellProbe::new()
                .executable(CLAUDE)
                .executable(HOOK_PATH)
                .answers(ShellMode::LoginInteractive, "command -v claude", CLAUDE)
                .version(CLAUDE, "2.1.330 (Claude Code)"),
            notices: RecordingNotices::default(),
            transcripts: InMemoryTranscripts::default(),
            locks: InMemoryLocks::default(),
            executor,
        }
    }

    fn ports(&self) -> LaunchPorts<'_> {
        LaunchPorts {
            categories: &self.store,
            sessions: &self.store,
            settings: &self.store,
            shell: &self.probe,
            notices: &self.notices,
            hook_binary: &FixedHookBinary,
            transcripts: &self.transcripts,
            locks: &self.locks,
        }
    }
}

fn environment() -> LaunchEnvironment {
    LaunchEnvironment {
        shell: PathBuf::from("/fake/zsh"),
        lithemux_home: None,
    }
}

fn target() -> SessionRef {
    SessionRef::parse("demo/ejecutora").unwrap()
}

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

// PLAN §0.3: the lock lasts as long as the process; docs/decisiones.md: pid inside
#[test]
fn the_session_is_locked_while_it_runs_and_free_afterwards() {
    let world = World::new();
    let id = world.executor.id.clone();
    let check = || assert_eq!(world.locks.holder(&id), Some(4242));
    let mut runner = RecordingRunner::exiting_with(0);
    runner.while_running = Some(&check);

    open_session(&world.ports(), &runner, &environment(), &target()).unwrap();

    assert_eq!(world.locks.holder(&id), None);
}

// PLAN §0.3: opening the same session twice fails
#[test]
fn a_session_that_is_already_open_cannot_be_opened_again() {
    let world = World::new();
    let runner = RecordingRunner::exiting_with(0);
    let mut held = world.locks.acquire(&world.executor.id).unwrap();
    held.record_pid(999);

    let second = open_session(&world.ports(), &runner, &environment(), &target());

    assert!(matches!(
        second,
        Err(LaunchError::AlreadyOpen { pid: Some(999) })
    ));
    assert_eq!(
        second.unwrap_err().to_string(),
        "the session is already open (pid 999)"
    );
    assert!(runner.plans.borrow().is_empty());
}

/// Notes whether the session was locked when the transcript was looked up.
struct TranscriptsSeenUnderLock<'a> {
    locks: &'a InMemoryLocks,
    session_id: &'a str,
    locked_when_asked: Cell<Option<bool>>,
}

impl TranscriptLocator for TranscriptsSeenUnderLock<'_> {
    fn exists(&self, _: &str) -> bool {
        self.locked_when_asked
            .set(Some(self.locks.holder(self.session_id).is_some()));
        false
    }
}

// PLAN §2.0 bug 5: whether to resume is decided with the lock already held,
// so two launches cannot both conclude that the conversation is new
#[test]
fn the_transcript_is_looked_up_with_the_session_already_locked() {
    let world = World::new();
    let runner = RecordingRunner::exiting_with(0);
    let transcripts = TranscriptsSeenUnderLock {
        locks: &world.locks,
        session_id: &world.executor.id,
        locked_when_asked: Cell::new(None),
    };
    let ports = LaunchPorts {
        transcripts: &transcripts,
        ..world.ports()
    };

    open_session(&ports, &runner, &environment(), &target()).unwrap();

    assert_eq!(transcripts.locked_when_asked.get(), Some(true));
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
    let mut new = new_session("demo/term", Kind::Child);
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
