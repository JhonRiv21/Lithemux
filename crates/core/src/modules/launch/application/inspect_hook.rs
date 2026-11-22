use crate::modules::launch::application::hook_inspection::HookInspection;
use crate::modules::launch::error::LaunchError;
use crate::modules::launch::ports::hook_binary::HookBinary;
use crate::modules::launch::ports::hook_probe::HookProbe;
use crate::modules::launch::ports::shell_probe::ShellProbe;

/// What `doctor` reports about the hook: whether a launch would find it and
/// whether it stays out of the way outside a lithemux session.
pub fn inspect_hook(
    hook_binary: &dyn HookBinary,
    shell: &dyn ShellProbe,
    probe: &dyn HookProbe,
) -> HookInspection {
    let path = hook_binary.path().and_then(|path| {
        if shell.is_executable(&path) {
            Ok(path)
        } else {
            Err(LaunchError::HookNotExecutable(path))
        }
    });
    let dry_run = path.as_ref().ok().map(|path| probe.dry_run(path));
    HookInspection { path, dry_run }
}
