use std::path::{Path, PathBuf};

use crate::modules::launch::domain::claude_plan::{ClaudeLaunch, claude_plan};
use crate::modules::launch::domain::hooks_settings::hooks_settings_json;
use crate::modules::launch::domain::test::fixtures::{
    CLAUDE_SESSION_ID, SESSION_ID, category, claude_session,
};
use crate::modules::session::Session;

const CLAUDE: &str = "/usr/local/bin/claude";
const HOOK: &str = "/opt/lithemux/bin/lithemux-hook";

fn plan(
    session: &Session,
    transcript_exists: bool,
    home: Option<&Path>,
) -> crate::modules::launch::domain::launch_plan::LaunchPlan {
    claude_plan(&ClaudeLaunch {
        category: &category(),
        session,
        claude_bin: Path::new(CLAUDE),
        hook_bin: Path::new(HOOK),
        transcript_exists,
        lithemux_home: home,
    })
    .unwrap()
}

// PLAN §0.3: no transcript yet -> --session-id
#[test]
fn a_conversation_without_transcript_starts_with_session_id() {
    let plan = plan(&claude_session(), false, None);
    assert_eq!(plan.args[0], "--session-id");
    assert_eq!(plan.args[1], CLAUDE_SESSION_ID);
    assert!(!plan.args.contains(&"--resume".to_string()));
}

// PLAN §0.3: transcript exists -> --resume, never --session-id
#[test]
fn a_conversation_with_transcript_is_resumed() {
    let plan = plan(&claude_session(), true, None);
    assert_eq!(plan.args[0], "--resume");
    assert_eq!(plan.args[1], CLAUDE_SESSION_ID);
    assert!(!plan.args.contains(&"--session-id".to_string()));
}

// PLAN §0.3: arguments always present
#[test]
fn the_exact_arguments_without_model() {
    let plan = plan(&claude_session(), false, None);
    assert_eq!(plan.program, PathBuf::from(CLAUDE));
    assert_eq!(
        plan.args,
        vec![
            "--session-id".to_string(),
            CLAUDE_SESSION_ID.to_string(),
            "-n".to_string(),
            "selektal-ejecutora".to_string(),
            "--settings".to_string(),
            hooks_settings_json(Path::new(HOOK)),
        ]
    );
}

// PLAN §0.3: --model only when the session defines one
#[test]
fn the_model_is_passed_only_when_defined() {
    let mut session = claude_session();
    session.model = Some("opus".to_string());
    let plan = plan(&session, true, None);
    assert_eq!(plan.args[6..], ["--model".to_string(), "opus".to_string()]);
    assert_eq!(plan.args.len(), 8);
}

// PLAN §0.3: child environment
#[test]
fn the_child_gets_the_lithemux_session_id() {
    let plan = plan(&claude_session(), false, None);
    assert_eq!(
        plan.env,
        vec![("LITHEMUX_SESSION_ID".to_string(), SESSION_ID.to_string())]
    );
}

// PLAN §0.3: LITHEMUX_HOME is forwarded only when it is set
#[test]
fn lithemux_home_is_forwarded_when_set() {
    let plan = plan(&claude_session(), false, Some(Path::new("/tmp/lx-home")));
    assert!(
        plan.env
            .contains(&("LITHEMUX_HOME".to_string(), "/tmp/lx-home".to_string()))
    );
}

// PLAN §0.2: cwd NULL means the category's root_dir
#[test]
fn the_working_directory_defaults_to_the_category_root() {
    assert_eq!(
        plan(&claude_session(), false, None).cwd,
        PathBuf::from("/work/selektal")
    );
    let mut session = claude_session();
    session.cwd = Some("/work/selektal/api".to_string());
    assert_eq!(
        plan(&session, false, None).cwd,
        PathBuf::from("/work/selektal/api")
    );
}

// PLAN §0.2: claude_session_id is what goes to --session-id/--resume
#[test]
fn a_claude_session_without_conversation_id_cannot_be_planned() {
    let mut session = claude_session();
    session.claude_session_id = None;
    let result = claude_plan(&ClaudeLaunch {
        category: &category(),
        session: &session,
        claude_bin: Path::new(CLAUDE),
        hook_bin: Path::new(HOOK),
        transcript_exists: false,
        lithemux_home: None,
    });
    assert!(result.is_err());
}
