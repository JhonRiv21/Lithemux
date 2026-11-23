use std::cell::Cell;

use lithemux_core::modules::launch::{
    LaunchError, LaunchPorts, SessionLocks, TranscriptLocator, open_session,
};

use crate::support::in_memory_launch_ports::{InMemoryLocks, RecordingRunner};

use crate::launch_world::{World, environment, target};

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
