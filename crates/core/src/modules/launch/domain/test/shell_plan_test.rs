use std::path::{Path, PathBuf};

use crate::modules::launch::domain::shell_plan::shell_plan;
use crate::modules::launch::domain::test::fixtures::{category, claude_session};
use crate::modules::session::Profile;

// PLAN §0.3: shell profile is `$SHELL -l` in the cwd, with no hooks and no context
#[test]
fn a_shell_session_is_a_login_shell_without_hooks() {
    let mut session = claude_session();
    session.profile = Profile::Shell;
    session.claude_session_id = None;
    let plan = shell_plan(&category(), &session, Path::new("/fake/zsh"), None);
    assert_eq!(plan.program, PathBuf::from("/fake/zsh"));
    assert_eq!(plan.args, vec!["-l".to_string()]);
    assert_eq!(plan.cwd, PathBuf::from("/work/selektal"));
    assert!(!plan.args.iter().any(|arg| arg.contains("hook")));
}
