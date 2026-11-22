use std::path::PathBuf;

use lithemux_core::modules::launch::{
    LaunchEnvironment, LaunchError, LaunchPorts, SessionLocks, ShellMode, prepare_launch,
};
use lithemux_core::modules::session::{Kind, Profile, Session, SessionRef, create_session};

use crate::support::fake_shell_probe::FakeShellProbe;
use crate::support::in_memory_launch_ports::{
    FixedHookBinary, HOOK_PATH, InMemoryLocks, InMemoryTranscripts,
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
                .answers(ShellMode::LoginInteractive, "command -v claude", CLAUDE),
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

// PLAN §2.1: the pieces of a launch, without running anything. The lock is
// taken and stays taken for as long as the prepared launch is kept
#[test]
fn a_prepared_launch_holds_the_lock_until_it_is_dropped() {
    let world = World::new();
    let id = world.executor.id.clone();

    let mut prepared = prepare_launch(&world.ports(), &environment(), &target()).unwrap();

    assert!(prepared.holds_a_lock());
    assert!(world.locks.holder(&id).is_some());
    assert_eq!(prepared.session().id, id);
    assert_eq!(prepared.plan().program, PathBuf::from(CLAUDE));
    prepared.started(5150);
    assert_eq!(world.locks.holder(&id), Some(5150));
    drop(prepared);
    assert_eq!(world.locks.holder(&id), None);
}

// PLAN §2.1: the command is decided with the lock taken, so it resumes when
// there is a conversation to resume
#[test]
fn a_prepared_launch_resumes_a_conversation_that_exists() {
    let world = World::new();
    let conversation = world.executor.claude_session_id.clone().unwrap();
    world
        .transcripts
        .existing
        .borrow_mut()
        .push(conversation.clone());

    let prepared = prepare_launch(&world.ports(), &environment(), &target()).unwrap();

    assert_eq!(
        prepared.plan().args[..2],
        ["--resume".to_string(), conversation]
    );
}

// PLAN §2.1: a session someone else has open is not prepared at all
#[test]
fn a_session_open_elsewhere_cannot_be_prepared() {
    let world = World::new();
    let mut held = world.locks.acquire(&world.executor.id).unwrap();
    held.record_pid(999);

    let prepared = prepare_launch(&world.ports(), &environment(), &target());

    assert!(matches!(
        prepared.err(),
        Some(LaunchError::AlreadyOpen { pid: Some(999) })
    ));
}

// PLAN §2.1: a shell session is prepared without a lock
#[test]
fn a_shell_session_is_prepared_without_a_lock() {
    let world = World::new();
    let mut shell = new_session("demo/term", Kind::Child);
    shell.profile = Profile::Shell;
    let (_, terminal) = create_session(&world.store, &world.store, shell, NOW).unwrap();
    let target = SessionRef::parse("demo/term").unwrap();

    let prepared = prepare_launch(&world.ports(), &environment(), &target).unwrap();

    assert!(!prepared.holds_a_lock());
    assert_eq!(world.locks.holder(&terminal.id), None);
    assert_eq!(prepared.plan().program, PathBuf::from("/fake/zsh"));
}
