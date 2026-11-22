use std::cell::Cell;
use std::path::PathBuf;

use lithemux_core::modules::launch::{
    HookDryRun, HookProbe, HookRunOutput, LaunchError, inspect_hook,
};

use crate::support::fake_shell_probe::FakeShellProbe;
use crate::support::in_memory_launch_ports::{FixedHookBinary, HOOK_PATH};

/// Answers a dry run without starting anything, and counts the calls.
struct CountingHookProbe {
    runs: Cell<usize>,
}

impl HookProbe for CountingHookProbe {
    fn dry_run(&self, _: &std::path::Path) -> HookDryRun {
        self.runs.set(self.runs.get() + 1);
        HookDryRun {
            elapsed: std::time::Duration::from_millis(3),
            outcome: Ok(HookRunOutput {
                succeeded: true,
                status: "exit status: 0".to_string(),
                stdout_bytes: 0,
                stderr_bytes: 0,
            }),
        }
    }
}

// PLAN §0.6 doctor + §2.0 point 15: the hook is found and tried in dry run
#[test]
fn doctor_tries_the_hook_it_would_launch_with() {
    let probe = CountingHookProbe { runs: Cell::new(0) };
    let shell = FakeShellProbe::new().executable(HOOK_PATH);

    let inspection = inspect_hook(&FixedHookBinary, &shell, &probe);

    assert_eq!(inspection.path.unwrap(), PathBuf::from(HOOK_PATH));
    assert!(inspection.dry_run.unwrap().was_silent_success());
    assert_eq!(probe.runs.get(), 1);
}

// PLAN §2.0 bug 9 + point 15: a hook that cannot run is reported, not run
#[test]
fn doctor_reports_a_hook_that_is_not_executable_without_running_it() {
    let probe = CountingHookProbe { runs: Cell::new(0) };

    let inspection = inspect_hook(&FixedHookBinary, &FakeShellProbe::new(), &probe);

    assert!(matches!(
        inspection.path,
        Err(LaunchError::HookNotExecutable(_))
    ));
    assert_eq!(inspection.dry_run, None);
    assert_eq!(probe.runs.get(), 0);
}

// PLAN §0.6 doctor: any output or a failure outside a session is a problem
#[test]
fn a_dry_run_is_clean_only_when_it_succeeds_in_silence() {
    let run = |succeeded, stdout_bytes, stderr_bytes| HookDryRun {
        elapsed: std::time::Duration::ZERO,
        outcome: Ok(HookRunOutput {
            succeeded,
            status: String::new(),
            stdout_bytes,
            stderr_bytes,
        }),
    };
    assert!(run(true, 0, 0).was_silent_success());
    assert!(!run(false, 0, 0).was_silent_success());
    assert!(!run(true, 1, 0).was_silent_success());
    assert!(!run(true, 0, 1).was_silent_success());
    let failed_to_start = HookDryRun {
        elapsed: std::time::Duration::ZERO,
        outcome: Err("not found".to_string()),
    };
    assert!(!failed_to_start.was_silent_success());
}
