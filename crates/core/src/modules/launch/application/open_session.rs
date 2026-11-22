use std::time::Duration;

use crate::modules::launch::application::launch_ports::LaunchPorts;
use crate::modules::launch::application::plan_launch::plan_launch;
use crate::modules::launch::domain::claude_version::parse_version_output;
use crate::modules::launch::domain::launch_environment::LaunchEnvironment;
use crate::modules::launch::error::LaunchError;
use crate::modules::session::{Profile, SessionRef, get_session, record_claude_version};

const VERSION_TIMEOUT: Duration = Duration::from_secs(2);

/// Runs the session and returns its exit code.
///
/// The lock is held here, by the process that waits, and is not handed to the
/// child: a descriptor inherited by `claude` would also reach anything
/// `claude` leaves running, and the session could then never be opened again.
pub fn open_session(
    ports: &LaunchPorts<'_>,
    environment: &LaunchEnvironment,
    target: &SessionRef,
) -> Result<i32, LaunchError> {
    let (category, session) = get_session(ports.categories, ports.sessions, target)?;
    if session.profile == Profile::Shell {
        // A plain shell has no conversation to protect.
        let plan = plan_launch(ports, environment, &category, &session)?;
        return ports.runner.run(&plan, &mut |_| {});
    }
    // Locked before the plan is made: whether to resume depends on the
    // transcript, and that answer is only safe while nobody else can start.
    let mut lock = ports.locks.acquire(&session.id)?;
    let plan = plan_launch(ports, environment, &category, &session)?;
    // Best effort: a missing version never blocks or delays the launch.
    if let Some(version) = ports
        .shell
        .version_output(&plan.program, VERSION_TIMEOUT)
        .as_deref()
        .and_then(parse_version_output)
    {
        let _ = record_claude_version(ports.sessions, &session.id, &version);
    }
    ports.runner.run(&plan, &mut |pid| lock.record_pid(pid))
}
