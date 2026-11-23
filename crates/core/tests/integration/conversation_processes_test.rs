//! PLAN §3.4: a conversation that runs in a process lithemux did not start
//! is found by its command line. The process is the fake `claude`.

use std::time::Instant;

use lithemux_core::modules::launch::{
    CONVERSATION_FLAGS, CommandLineConversations, ConversationProcesses, conversation_in,
};
use lithemux_core::platform::process::memory_use::MemoryMeter;

use crate::sandbox::fake_claude_elsewhere;

const ID: &str = "6f1d2c3b-4a59-4e8f-9b7a-0c1d2e3f4a5b";
const OTHER: &str = "7a2e3d4c-5b6a-4f9e-8c7b-1d2e3f4a5b6c";
/// Each test its own: they run at the same time.
const SWEPT: &str = "8b3f4e5d-6c7b-4a0f-9d8c-2e3f4a5b6c7d";

// PLAN §3.4: «si un proceso vivo tiene ese uuid en su argv, la sesión se
// trata como abierta en otro lado (pid N)»
#[test]
fn a_claude_started_elsewhere_is_found_with_its_pid() {
    let dir = tempfile::tempdir().unwrap();
    let mut child = fake_claude_elsewhere(dir.path(), &["--resume", ID]);

    let found = CommandLineConversations.holder_of(ID);
    let other = CommandLineConversations.holder_of(OTHER);

    child.kill().unwrap();
    child.wait().unwrap();
    // The fake itself, or a subshell of it caught before it runs its
    // command, which shows the same command line.
    assert!(found.is_some());
    assert_eq!(other, None);
    assert_eq!(CommandLineConversations.holder_of(ID), None);
}

// PLAN §3.4: the card says it on its own, from the sweep the memory meter
// already makes, and forgets it once the process is gone
#[test]
fn the_memory_meter_keeps_who_runs_a_conversation_while_it_runs() {
    let dir = tempfile::tempdir().unwrap();
    let mut child = fake_claude_elsewhere(dir.path(), &["--session-id", SWEPT]);
    let mut meter = MemoryMeter::watching(&CONVERSATION_FLAGS);

    meter.measure(&[], Instant::now());
    let seen: Vec<_> = meter
        .commands()
        .iter()
        .filter(|line| conversation_in(&line.args) == Some(SWEPT))
        .map(|line| line.pid)
        .collect();
    child.kill().unwrap();
    child.wait().unwrap();
    meter.measure(&[], Instant::now());

    // A shell forking a subshell gives it its own command line for a
    // moment: that one may be seen too, and it is the same conversation.
    assert!(
        seen.contains(&child.id()),
        "{seen:?} without {}",
        child.id()
    );
    assert!(meter.commands().iter().all(|line| line.pid != child.id()));
}
