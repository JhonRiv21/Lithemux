use std::path::{Path, PathBuf};

use crate::modules::category::Category;
use crate::modules::launch::domain::child_env::child_env;
use crate::modules::launch::domain::hooks_settings::hooks_settings_json;
use crate::modules::launch::domain::launch_plan::LaunchPlan;
use crate::modules::launch::error::LaunchError;
use crate::modules::session::Session;

pub struct ClaudeLaunch<'a> {
    pub category: &'a Category,
    pub session: &'a Session,
    pub claude_bin: &'a Path,
    pub hook_bin: &'a Path,
    pub transcript_exists: bool,
    pub lithemux_home: Option<&'a Path>,
}

/// `--resume` when the transcript exists, `--session-id` otherwise: a
/// conversation that exists is never started over.
pub fn claude_plan(launch: &ClaudeLaunch<'_>) -> Result<LaunchPlan, LaunchError> {
    let ClaudeLaunch {
        category, session, ..
    } = launch;
    let name = session.messaging_name(category);
    let claude_session_id = session
        .claude_session_id
        .as_deref()
        .ok_or_else(|| LaunchError::MissingClaudeSessionId(name.clone()))?;
    let conversation_flag = if launch.transcript_exists {
        "--resume"
    } else {
        "--session-id"
    };
    let mut args = vec![
        conversation_flag.to_string(),
        claude_session_id.to_string(),
        "-n".to_string(),
        name,
        "--settings".to_string(),
        hooks_settings_json(launch.hook_bin),
    ];
    if let Some(model) = session.model.as_deref() {
        args.push("--model".to_string());
        args.push(model.to_string());
    }
    Ok(LaunchPlan {
        program: launch.claude_bin.to_path_buf(),
        args,
        env: child_env(session, launch.lithemux_home),
        cwd: PathBuf::from(session.effective_cwd(category)),
    })
}
