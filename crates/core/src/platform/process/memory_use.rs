//! How much memory a session uses: its program and everything below it.
//! Read from the kernel's process tables; nothing here touches the network.
//!
//! Finding what is below a program means listing every process with its
//! parent (about 5 ms on a Mac with 600 processes), so the trees are mapped
//! again only every `REMAP_EVERY`. Reading the memory of a known process
//! costs microseconds, and is all that each measure does for the processes
//! of the trees.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

/// How often the trees are mapped again. A process started meanwhile is
/// counted from the next map on.
pub const REMAP_EVERY: Duration = Duration::from_secs(30);
/// A runaway tree is not followed forever.
const MAX_TREE: usize = 512;

/// `root` and every process below it in `parents` (pairs of `(pid,
/// parent)`), children of children included.
pub fn descendants(parents: &[(u32, u32)], root: u32) -> Vec<u32> {
    let mut tree = vec![root];
    let mut next = 0;
    while next < tree.len() && tree.len() < MAX_TREE {
        let parent = tree[next];
        for (pid, _) in parents.iter().filter(|(_, of)| *of == parent) {
            if !tree.contains(pid) {
                tree.push(*pid);
            }
        }
        next += 1;
    }
    tree
}

/// Measures the memory of process trees, again and again. It keeps the
/// trees it mapped and the kernel handle between measures.
pub struct MemoryMeter {
    system: Option<System>,
    trees: HashMap<u32, Vec<u32>>,
    mapped_at: Option<Instant>,
}

impl Default for MemoryMeter {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryMeter {
    pub fn new() -> Self {
        MemoryMeter {
            system: System::new().ok(),
            trees: HashMap::new(),
            mapped_at: None,
        }
    }

    /// For each of `roots` that still runs, the resident memory of its whole
    /// tree, in bytes.
    pub fn measure(&mut self, roots: &[u32], now: Instant) -> HashMap<u32, u64> {
        let is_stale = self
            .mapped_at
            .is_none_or(|at| now.duration_since(at) >= REMAP_EVERY);
        let has_a_new_root = roots.iter().any(|root| !self.trees.contains_key(root));
        if is_stale || has_a_new_root {
            self.map(roots);
            self.mapped_at = Some(now);
        }
        let Some(system) = self.system.as_mut() else {
            return HashMap::new();
        };
        let pids: Vec<Pid> = roots
            .iter()
            .filter_map(|root| self.trees.get(root))
            .flatten()
            .map(|pid| Pid::from_u32(*pid))
            .collect();
        system.refresh_processes_specifics(
            ProcessesToUpdate::Some(&pids),
            true,
            ProcessRefreshKind::nothing().with_memory(),
        );
        let memory_of = |pid: &u32| {
            let process = system.process(Pid::from_u32(*pid));
            process.map_or(0, sysinfo::Process::memory)
        };
        // A root that is gone is not measured: «0 MB» would be a figure
        // for something that no longer exists.
        roots
            .iter()
            .filter(|root| system.process(Pid::from_u32(**root)).is_some())
            .map(|root| {
                let tree = self.trees.get(root).map_or(&[][..], Vec::as_slice);
                (*root, tree.iter().map(memory_of).sum())
            })
            .collect()
    }

    fn map(&mut self, roots: &[u32]) {
        let Some(system) = self.system.as_mut() else {
            return;
        };
        system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing(),
        );
        let parents: Vec<(u32, u32)> = system
            .processes()
            .iter()
            .filter_map(|(pid, process)| Some((pid.as_u32(), process.parent()?.as_u32())))
            .collect();
        self.trees = roots
            .iter()
            .map(|root| (*root, descendants(&parents, *root)))
            .collect();
    }
}
