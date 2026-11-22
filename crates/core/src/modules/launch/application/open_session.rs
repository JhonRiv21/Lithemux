use crate::modules::launch::application::launch_ports::LaunchPorts;
use crate::modules::launch::application::prepare_launch::prepare_launch;
use crate::modules::launch::domain::launch_environment::LaunchEnvironment;
use crate::modules::launch::error::LaunchError;
use crate::modules::launch::ports::session_runner::SessionRunner;
use crate::modules::session::SessionRef;

/// Runs the session with `runner`, waits for it and returns its exit code.
/// This is how the CLI opens a session.
///
/// The lock is held here, by the process that waits, and is not handed to the
/// child: a descriptor inherited by `claude` would also reach anything
/// `claude` leaves running, and the session could then never be opened again.
pub fn open_session(
    ports: &LaunchPorts<'_>,
    runner: &dyn SessionRunner,
    environment: &LaunchEnvironment,
    target: &SessionRef,
) -> Result<i32, LaunchError> {
    let mut prepared = prepare_launch(ports, environment, target)?;
    let plan = prepared.plan().clone();
    runner.run(&plan, &mut |pid| prepared.started(pid))
}
