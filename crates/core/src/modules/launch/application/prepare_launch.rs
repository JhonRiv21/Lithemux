use std::time::Duration;

use crate::modules::launch::application::launch_ports::LaunchPorts;
use crate::modules::launch::application::plan_launch::plan_launch;
use crate::modules::launch::application::prepared_launch::PreparedLaunch;
use crate::modules::launch::domain::claude_version::parse_version_output;
use crate::modules::launch::domain::launch_environment::LaunchEnvironment;
use crate::modules::launch::error::LaunchError;
use crate::modules::session::{Profile, SessionRef, get_session, record_claude_version};

const VERSION_TIMEOUT: Duration = Duration::from_secs(2);

/// Everything short of starting the process, the same for the CLI and the
/// app: takes the session's lock and then decides the command.
///
/// The lock comes first. Whether to resume depends on the transcript, and
/// that answer is only safe while nobody else can start the session. It
/// fails with `LaunchError::AlreadyOpen` when someone else has it.
pub fn prepare_launch(
    ports: &LaunchPorts<'_>,
    environment: &LaunchEnvironment,
    target: &SessionRef,
) -> Result<PreparedLaunch, LaunchError> {
    let (category, session) = get_session(ports.categories, ports.sessions, target)?;
    if session.profile == Profile::Shell {
        // A plain shell has no conversation to protect.
        let plan = plan_launch(ports, environment, &category, &session)?;
        return Ok(PreparedLaunch::new(category, session, plan, None));
    }
    let lock = ports.locks.acquire(&session.id)?;
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
    Ok(PreparedLaunch::new(category, session, plan, Some(lock)))
}
