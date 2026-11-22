//! A category with a coordinator and one more session, the `ejecutora`,
//! whose hooks fire through `handle_hook_event` with ports in memory.

use lithemux_core::modules::context::ContextPorts;
use lithemux_core::modules::hook_event::{HookPorts, handle_hook_event};
use lithemux_core::modules::session::{Kind, Session, find_session_by_id};

use crate::support::in_memory_store::{InMemoryBoards, InMemoryStore};
use crate::support::recording_ports::{RecordingLog, RecordingNotifier};
use crate::support::sample_data::{add_category, add_session};

pub struct World {
    pub store: InMemoryStore,
    pub boards: InMemoryBoards,
    pub log: RecordingLog,
    pub notifier: RecordingNotifier,
    pub executor: Session,
}

impl World {
    pub fn new() -> Self {
        let (store, boards) = (InMemoryStore::new(), InMemoryBoards::default());
        add_category(&store, &boards, "demo");
        add_session(&store, &store, "demo/planificadora", Kind::Coordinator);
        let executor = add_session(&store, &store, "demo/ejecutora", Kind::Member);
        World {
            store,
            boards,
            log: RecordingLog::default(),
            notifier: RecordingNotifier::default(),
            executor,
        }
    }

    pub fn fire(&self, subcommand: &str, stdin: &str, now: i64) -> Option<String> {
        let ports = HookPorts {
            context: ContextPorts {
                categories: &self.store,
                sessions: &self.store,
                settings: &self.store,
                boards: &self.boards,
            },
            log: &self.log,
            notifier: &self.notifier,
        };
        handle_hook_event(&ports, &self.executor.id, subcommand, stdin, now)
    }

    pub fn executor_now(&self) -> Session {
        find_session_by_id(&self.store, &self.store, &self.executor.id)
            .unwrap()
            .unwrap()
            .1
    }
}
