use std::ffi::OsString;

use crate::terminal_view::ShellCommand;
use crate::terminal_view::child_env::child_env;

fn inherited(names: &[&str]) -> impl Iterator<Item = (OsString, OsString)> {
    names
        .iter()
        .map(|name| (OsString::from(name), OsString::from("from-parent")))
        .collect::<Vec<_>>()
        .into_iter()
}

fn names(env: &[(OsString, OsString)]) -> Vec<&str> {
    env.iter().filter_map(|(name, _)| name.to_str()).collect()
}

fn value<'a>(env: &'a [(OsString, OsString)], name: &str) -> Option<&'a str> {
    let found = env.iter().find(|(existing, _)| existing == name)?;
    found.1.to_str()
}

// PLAN §2.0 point 19: the app was started from a Claude Code session in a
// cmux tab, and the `claude` inside did not save its transcript
#[test]
fn a_program_does_not_inherit_the_marks_of_the_session_the_app_runs_in() {
    let env = child_env(
        &ShellCommand::new("/fake/program"),
        inherited(&[
            "HOME",
            "CLAUDECODE",
            "CLAUDE_CODE_CHILD_SESSION",
            "CLAUDE_CODE_ENTRYPOINT",
            "CLAUDE_CODE_SSE_PORT",
            "CMUX_SURFACE_ID",
            "CMUX_SOCKET_PATH",
        ]),
    );

    let names = names(&env);
    assert!(names.contains(&"HOME"));
    for name in &names {
        assert!(
            !name.starts_with("CMUX_") && !name.starts_with("CLAUDE"),
            "{name}"
        );
    }
}

// PLAN §2.0 point 19: "no se quitan las variables de configuración"
#[test]
fn what_the_user_configured_on_purpose_gets_through() {
    let kept = [
        "CLAUDE_CODE_MAX_OUTPUT_TOKENS",
        "CLAUDE_CONFIG_DIR",
        "ANTHROPIC_MODEL",
        "PATH",
    ];

    let env = child_env(&ShellCommand::new("/fake/program"), inherited(&kept));

    for name in kept {
        assert_eq!(value(&env, name), Some("from-parent"), "{name}");
    }
}

// PLAN §1.2 test 1: the terminal names itself, whatever it was started from
#[test]
fn the_terminal_overrides_what_the_parent_said_about_its_own_terminal() {
    let env = child_env(
        &ShellCommand::new("/fake/program"),
        inherited(&["TERM", "TERM_PROGRAM", "COLORTERM"]),
    );

    assert_eq!(value(&env, "TERM"), Some("xterm-256color"));
    assert_eq!(value(&env, "TERM_PROGRAM"), Some("lithemux"));
    assert_eq!(value(&env, "COLORTERM"), Some("truecolor"));
    assert_eq!(env.len(), 3);
}

// PLAN §2.1: what the launcher asks for wins, and a clean start stays clean
#[test]
fn the_command_has_the_last_word_and_can_start_from_nothing() {
    let mut command = ShellCommand::new("/fake/program");
    command.inherit_env = false;
    command.env.push(("TERM".to_string(), "dumb".to_string()));
    command
        .env
        .push(("LITHEMUX_SESSION_ID".to_string(), "abc".to_string()));

    let env = child_env(&command, inherited(&["HOME", "PATH"]));

    assert_eq!(value(&env, "TERM"), Some("dumb"));
    assert_eq!(value(&env, "LITHEMUX_SESSION_ID"), Some("abc"));
    assert_eq!(value(&env, "HOME"), None);
}
