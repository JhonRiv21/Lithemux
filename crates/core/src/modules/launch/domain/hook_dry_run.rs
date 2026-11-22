use std::time::Duration;

/// What running the hook outside a lithemux session did. There it has to be
/// a no-op: exit 0 and not a byte of output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookDryRun {
    pub elapsed: Duration,
    pub outcome: Result<HookRunOutput, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookRunOutput {
    pub succeeded: bool,
    /// How the process ended, for people.
    pub status: String,
    pub stdout_bytes: usize,
    pub stderr_bytes: usize,
}

impl HookDryRun {
    pub fn was_silent_success(&self) -> bool {
        self.outcome
            .as_ref()
            .is_ok_and(|run| run.succeeded && run.stdout_bytes == 0 && run.stderr_bytes == 0)
    }
}
