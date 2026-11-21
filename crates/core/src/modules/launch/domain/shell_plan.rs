use std::path::{Path, PathBuf};

use crate::modules::category::Category;
use crate::modules::launch::domain::child_env::child_env;
use crate::modules::launch::domain::launch_plan::LaunchPlan;
use crate::modules::session::Session;

/// A login shell in the session's directory, with no hooks and no context.
pub fn shell_plan(
    category: &Category,
    session: &Session,
    shell: &Path,
    lithemux_home: Option<&Path>,
) -> LaunchPlan {
    LaunchPlan {
        program: shell.to_path_buf(),
        args: vec!["-l".to_string()],
        env: child_env(session, lithemux_home),
        cwd: PathBuf::from(session.effective_cwd(category)),
    }
}
