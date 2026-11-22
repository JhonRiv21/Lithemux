use std::path::Path;
use std::process::Stdio;
use std::time::Instant;

use crate::modules::launch::domain::hook_dry_run::{HookDryRun, HookRunOutput};
use crate::modules::launch::ports::hook_probe::HookProbe;
use crate::platform::paths::home::SESSION_ENV;
use crate::platform::process::session_markers::command_without_session_markers;

const DRY_RUN_EVENT: &str = "stop";

pub struct ProcessHookProbe;

impl HookProbe for ProcessHookProbe {
    fn dry_run(&self, hook_bin: &Path) -> HookDryRun {
        let started = Instant::now();
        let output = command_without_session_markers(hook_bin)
            .arg(DRY_RUN_EVENT)
            .env_remove(SESSION_ENV)
            .stdin(Stdio::null())
            .output();
        HookDryRun {
            elapsed: started.elapsed(),
            outcome: output
                .map(|output| HookRunOutput {
                    succeeded: output.status.success(),
                    status: output.status.to_string(),
                    stdout_bytes: output.stdout.len(),
                    stderr_bytes: output.stderr.len(),
                })
                .map_err(|error| error.to_string()),
        }
    }
}
