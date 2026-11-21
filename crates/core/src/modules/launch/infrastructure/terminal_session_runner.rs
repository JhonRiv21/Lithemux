use crate::modules::launch::domain::launch_plan::LaunchPlan;
use crate::modules::launch::error::LaunchError;
use crate::modules::launch::ports::session_runner::SessionRunner;
use crate::platform::process::supervise::{ChildSpec, run_to_exit};

/// Runs the session as a child that inherits the current terminal.
pub struct TerminalSessionRunner;

impl SessionRunner for TerminalSessionRunner {
    fn run(&self, plan: &LaunchPlan, on_started: &mut dyn FnMut(u32)) -> Result<i32, LaunchError> {
        if !plan.cwd.is_dir() {
            return Err(LaunchError::MissingCwd(plan.cwd.clone()));
        }
        let spec = ChildSpec {
            program: &plan.program,
            args: &plan.args,
            env: &plan.env,
            cwd: &plan.cwd,
        };
        run_to_exit(&spec, on_started).map_err(|source| LaunchError::Spawn {
            program: plan.program.clone(),
            source,
        })
    }
}
