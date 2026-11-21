use crate::modules::launch::domain::launch_plan::LaunchPlan;
use crate::modules::launch::error::LaunchError;

pub trait SessionRunner {
    /// Runs the plan to completion and returns its exit code. `on_started`
    /// receives the pid as soon as the process exists.
    fn run(&self, plan: &LaunchPlan, on_started: &mut dyn FnMut(u32)) -> Result<i32, LaunchError>;
}
