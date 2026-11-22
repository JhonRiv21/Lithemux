use std::path::Path;

use crate::modules::launch::domain::hook_dry_run::HookDryRun;

pub trait HookProbe {
    /// Runs the hook the way Claude Code would, but outside any lithemux
    /// session.
    fn dry_run(&self, hook_bin: &Path) -> HookDryRun;
}
