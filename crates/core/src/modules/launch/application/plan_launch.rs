use crate::modules::category::Category;
use crate::modules::launch::application::launch_ports::LaunchPorts;
use crate::modules::launch::application::resolve_claude::resolve_claude;
use crate::modules::launch::domain::claude_plan::{ClaudeLaunch, claude_plan};
use crate::modules::launch::domain::launch_environment::LaunchEnvironment;
use crate::modules::launch::domain::launch_plan::LaunchPlan;
use crate::modules::launch::domain::shell_plan::shell_plan;
use crate::modules::launch::error::LaunchError;
use crate::modules::session::{Profile, Session};

/// Builds the command for a stored session. Nothing is started here.
pub fn plan_launch(
    ports: &LaunchPorts<'_>,
    environment: &LaunchEnvironment,
    category: &Category,
    session: &Session,
) -> Result<LaunchPlan, LaunchError> {
    let lithemux_home = environment.lithemux_home.as_deref();
    if session.profile == Profile::Shell {
        return Ok(shell_plan(
            category,
            session,
            &environment.shell,
            lithemux_home,
        ));
    }
    let claude = resolve_claude(ports.settings, ports.shell)?;
    if let Some(warning) = claude.warning.as_deref() {
        ports.notices.warn(warning);
    }
    let hook_bin = ports.hook_binary.path()?;
    if !ports.shell.is_executable(&hook_bin) {
        return Err(LaunchError::HookNotExecutable(hook_bin));
    }
    let transcript_exists = session
        .claude_session_id
        .as_deref()
        .is_some_and(|id| ports.transcripts.exists(id));
    claude_plan(&ClaudeLaunch {
        category,
        session,
        claude_bin: &claude.path,
        hook_bin: &hook_bin,
        transcript_exists,
        lithemux_home,
    })
}
