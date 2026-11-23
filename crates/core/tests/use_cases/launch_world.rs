//! The launch use cases' world: a category, a session and in-memory ports.

use std::path::PathBuf;

use lithemux_core::modules::launch::{LaunchEnvironment, LaunchPorts, ShellMode};
use lithemux_core::modules::session::{Kind, Session, SessionRef};

use crate::support::fake_shell_probe::FakeShellProbe;
use crate::support::in_memory_launch_ports::{
    FixedHookBinary, HOOK_PATH, InMemoryConversations, InMemoryLocks, InMemoryTranscripts,
};
use crate::support::in_memory_store::{InMemoryBoards, InMemoryStore};
use crate::support::recording_ports::RecordingNotices;
use crate::support::sample_data::{add_category, add_session};

pub const CLAUDE: &str = "/Users/x/.local/bin/claude";

pub struct World {
    pub store: InMemoryStore,
    pub probe: FakeShellProbe,
    pub notices: RecordingNotices,
    pub transcripts: InMemoryTranscripts,
    pub locks: InMemoryLocks,
    pub conversations: InMemoryConversations,
    pub executor: Session,
}

impl World {
    pub fn new() -> Self {
        let store = InMemoryStore::new();
        add_category(&store, &InMemoryBoards::default(), "demo");
        let executor = add_session(&store, &store, "demo/ejecutora", Kind::Member);
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
            conversations: InMemoryConversations::default(),
            executor,
        }
    }

    pub fn ports(&self) -> LaunchPorts<'_> {
        LaunchPorts {
            categories: &self.store,
            sessions: &self.store,
            settings: &self.store,
            shell: &self.probe,
            notices: &self.notices,
            hook_binary: &FixedHookBinary,
            transcripts: &self.transcripts,
            locks: &self.locks,
            conversations: &self.conversations,
        }
    }
}

pub fn environment() -> LaunchEnvironment {
    LaunchEnvironment {
        shell: PathBuf::from("/fake/zsh"),
        lithemux_home: None,
    }
}

pub fn target() -> SessionRef {
    SessionRef::parse("demo/ejecutora").unwrap()
}
