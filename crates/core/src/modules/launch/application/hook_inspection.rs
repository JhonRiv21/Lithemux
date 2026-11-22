use std::path::PathBuf;

use crate::modules::launch::domain::hook_dry_run::HookDryRun;
use crate::modules::launch::error::LaunchError;

#[derive(Debug)]
pub struct HookInspection {
    /// Where the hook is, or why a launch could not use it.
    pub path: Result<PathBuf, LaunchError>,
    /// Only when there is a hook to run.
    pub dry_run: Option<HookDryRun>,
}
