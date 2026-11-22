//! The TCP ports a program and everything it started listen on. Read from
//! the kernel's process tables (macOS libproc, through `listeners` and
//! `sysinfo`); nothing here touches the network.

use std::collections::{BTreeSet, HashMap};

use listeners::{Protocol, SocketState};
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

/// A chain of parents longer than this is not followed.
const MAX_DEPTH: usize = 64;

/// Asks for the parent of one process at a time and remembers the answers:
/// only the few processes that listen, and what is above them, are looked
/// at, never every process of the machine.
struct Parents {
    system: Option<System>,
    known: HashMap<u32, Option<u32>>,
}

impl Parents {
    fn new() -> Self {
        Parents {
            system: System::new().ok(),
            known: HashMap::new(),
        }
    }

    fn of(&mut self, pid: u32) -> Option<u32> {
        if let Some(parent) = self.known.get(&pid) {
            return *parent;
        }
        let parent = self.system.as_mut().and_then(|system| {
            let asked = Pid::from_u32(pid);
            system.refresh_processes_specifics(
                ProcessesToUpdate::Some(&[asked]),
                false,
                ProcessRefreshKind::nothing(),
            );
            Some(system.process(asked)?.parent()?.as_u32())
        });
        self.known.insert(pid, parent);
        parent
    }

    /// The one of `roots` that `pid` is, or descends from.
    fn root_of(&mut self, pid: u32, roots: &[u32]) -> Option<u32> {
        let mut current = pid;
        for _ in 0..MAX_DEPTH {
            if roots.contains(&current) {
                return Some(current);
            }
            current = self.of(current)?;
        }
        None
    }
}

/// For each of `roots`, the ports its tree listens on, in order and without
/// repeats. One pass over the machine's listening sockets serves them all;
/// from each listener the way up to its root is a few lookups.
pub fn listening_ports(roots: &[u32]) -> HashMap<u32, Vec<u16>> {
    let mut found: HashMap<u32, BTreeSet<u16>> =
        roots.iter().map(|root| (*root, BTreeSet::new())).collect();
    if roots.is_empty() {
        return HashMap::new();
    }
    let mut parents = Parents::new();
    for listener in listeners::get_all().unwrap_or_default() {
        if listener.protocol != Protocol::TCP || listener.state != SocketState::Listen {
            continue;
        }
        if let Some(root) = parents.root_of(listener.process.pid, roots)
            && let Some(ports) = found.get_mut(&root)
        {
            ports.insert(listener.socket.port());
        }
    }
    found
        .into_iter()
        .map(|(root, ports)| (root, ports.into_iter().collect()))
        .collect()
}
