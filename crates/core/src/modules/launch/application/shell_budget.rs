use std::cell::Cell;
use std::time::Duration;

use crate::modules::launch::ports::shell_probe::{ShellMode, ShellProbe};

/// The time one operation may spend waiting for the user's shell, shared by
/// every question it asks. A shell whose profile hangs fails each question
/// the same way; without a shared limit each one would wait in full. Time
/// is read from the probe's clock, so it can be a made-up one.
pub struct ShellBudget {
    remaining: Cell<Duration>,
}

impl ShellBudget {
    pub fn new(total: Duration) -> Self {
        ShellBudget {
            remaining: Cell::new(total),
        }
    }

    /// Asks the shell, waiting at most `at_most` and never more than what is
    /// left. The question is charged the time it took, answered or not; with
    /// nothing left, the shell is not asked at all.
    pub fn ask(
        &self,
        probe: &dyn ShellProbe,
        mode: ShellMode,
        script: &str,
        at_most: Duration,
    ) -> Option<String> {
        let wait = at_most.min(self.remaining.get());
        if wait.is_zero() {
            return None;
        }
        let asked_at = probe.now();
        let answer = probe.run(mode, script, wait);
        let took = probe.now().saturating_duration_since(asked_at).min(wait);
        self.remaining.set(self.remaining.get() - took);
        answer
    }
}
