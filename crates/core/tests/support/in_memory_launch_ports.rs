use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use lithemux_core::modules::launch::{
    HeldSessionLock, HookBinary, LaunchError, LaunchPlan, SessionLocks, SessionRunner,
    TranscriptLocator,
};

pub const HOOK_PATH: &str = "/opt/lithemux/bin/lithemux-hook";

pub struct FixedHookBinary;

impl HookBinary for FixedHookBinary {
    fn path(&self) -> Result<PathBuf, LaunchError> {
        Ok(PathBuf::from(HOOK_PATH))
    }
}

/// A hook binary at a path the test chose.
pub struct HookBinaryAt(pub PathBuf);

impl HookBinary for HookBinaryAt {
    fn path(&self) -> Result<PathBuf, LaunchError> {
        Ok(self.0.clone())
    }
}

// Same contract as ClaudeProjectsTranscriptLocator.
#[derive(Default)]
pub struct InMemoryTranscripts {
    pub existing: RefCell<Vec<String>>,
}

impl TranscriptLocator for InMemoryTranscripts {
    fn exists(&self, claude_session_id: &str) -> bool {
        self.existing
            .borrow()
            .iter()
            .any(|id| id == claude_session_id)
    }
}

// Same contract as FlockSessionLocks: exclusive per session, released on drop.
#[derive(Default)]
pub struct InMemoryLocks {
    held: Arc<Mutex<HashMap<String, u32>>>,
}

struct HeldInMemory {
    session_id: String,
    held: Arc<Mutex<HashMap<String, u32>>>,
}

impl HeldSessionLock for HeldInMemory {
    fn record_pid(&mut self, pid: u32) {
        self.held
            .lock()
            .unwrap()
            .insert(self.session_id.clone(), pid);
    }
}

impl Drop for HeldInMemory {
    fn drop(&mut self) {
        self.held.lock().unwrap().remove(&self.session_id);
    }
}

impl SessionLocks for InMemoryLocks {
    fn acquire(&self, session_id: &str) -> Result<Box<dyn HeldSessionLock>, LaunchError> {
        if let Some(pid) = self.holder(session_id) {
            return Err(LaunchError::AlreadyOpen {
                pid: (pid != 0).then_some(pid),
            });
        }
        self.held.lock().unwrap().insert(session_id.to_string(), 0);
        Ok(Box::new(HeldInMemory {
            session_id: session_id.to_string(),
            held: Arc::clone(&self.held),
        }))
    }

    fn holder(&self, session_id: &str) -> Option<u32> {
        self.held.lock().unwrap().get(session_id).copied()
    }
}

/// Pretends to run the plan: reports a pid, records the plan and returns the
/// configured exit code. `while_running` runs while the session is "alive".
pub struct RecordingRunner<'a> {
    pub pid: u32,
    pub exit_code: i32,
    pub plans: RefCell<Vec<LaunchPlan>>,
    pub while_running: Option<&'a dyn Fn()>,
}

impl RecordingRunner<'_> {
    pub fn exiting_with(exit_code: i32) -> Self {
        RecordingRunner {
            pid: 4242,
            exit_code,
            plans: RefCell::new(Vec::new()),
            while_running: None,
        }
    }
}

impl SessionRunner for RecordingRunner<'_> {
    fn run(&self, plan: &LaunchPlan, on_started: &mut dyn FnMut(u32)) -> Result<i32, LaunchError> {
        self.plans.borrow_mut().push(plan.clone());
        on_started(self.pid);
        if let Some(check) = self.while_running {
            check();
        }
        Ok(self.exit_code)
    }
}
