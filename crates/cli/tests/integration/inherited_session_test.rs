use std::fs;

use crate::sandbox::Sandbox;

/// What a tab of cmux running Claude Code exports, as seen on a real machine.
const MARKERS: [&str; 9] = [
    "CLAUDECODE",
    "CLAUDE_CODE_CHILD_SESSION",
    "CLAUDE_CODE_ENTRYPOINT",
    "CLAUDE_CODE_SSE_PORT",
    "CLAUDE_CODE_SESSION_ID",
    "CLAUDE_CODE_MESSAGING_SOCKET",
    "CLAUDE_PID",
    "CMUX_SURFACE_ID",
    "CMUX_CLAUDE_PID",
];

/// What the user sets on purpose; it has to get through.
const USER_SETTINGS: [&str; 2] = ["CLAUDE_CODE_MAX_OUTPUT_TOKENS", "CLAUDE_CONFIG_DIR"];

fn open_inside_another_session(sandbox: &Sandbox, target: &str, stdin: &str) {
    let mut command = sandbox.lithemux(&["open", target]);
    for name in MARKERS {
        command.env(name, "1");
    }
    command.env("CLAUDE_CODE_MAX_OUTPUT_TOKENS", "9000");
    command.write_stdin(stdin).assert().success();
}

fn assert_clean(names: &str) {
    let names: Vec<&str> = names.lines().collect();
    for marker in MARKERS {
        assert!(!names.contains(&marker), "{marker} reached the child");
    }
    for setting in USER_SETTINGS {
        assert!(names.contains(&setting), "{setting} was lost");
    }
}

// PLAN §2.0 point 19: a `claude` that inherits the markers of another
// session believes it is nested and does not save its transcript
#[test]
fn claude_does_not_inherit_the_markers_of_the_session_that_launched_it() {
    let sandbox = Sandbox::new().with_demo();
    sandbox.use_fake_claude();

    open_inside_another_session(&sandbox, "demo/ejecutora", "");

    assert_clean(&fs::read_to_string(sandbox.home().join("fake-claude.env")).unwrap());
}

// PLAN §2.0 point 19: the same for a shell session, where the user may
// start `claude` by hand
#[test]
fn a_shell_session_does_not_inherit_the_markers_either() {
    let sandbox = Sandbox::new().with_demo();
    sandbox.run(&[
        "new",
        "demo/term",
        "--role",
        "Terminal",
        "--profile",
        "shell",
    ]);

    let record = "env | cut -d= -f1 > \"$LITHEMUX_HOME/shell.env\"\n";
    open_inside_another_session(&sandbox, "demo/term", record);

    assert_clean(&fs::read_to_string(sandbox.home().join("shell.env")).unwrap());
}

fn value_of(environment: &str, name: &str) -> Option<String> {
    environment
        .lines()
        .find_map(|line| line.strip_prefix(&format!("{name}=")))
        .map(str::to_string)
}

fn claude_environment_after(extra: &[(&str, &str)]) -> String {
    let sandbox = Sandbox::new().with_demo();
    sandbox.use_fake_claude();
    let mut command = sandbox.lithemux(&["open", "demo/ejecutora"]);
    for (name, value) in extra {
        command.env(name, value);
    }
    command.assert().success();
    fs::read_to_string(sandbox.home().join("fake-claude.environment")).unwrap()
}

// Audit I5: what Claude Code sets for its own tools is not for a session
// the user works in. With GIT_EDITOR=true, `git commit` without -m aborts
#[test]
fn what_claude_code_sets_for_its_tools_does_not_reach_the_session() {
    let environment = claude_environment_after(&[
        ("CLAUDECODE", "1"),
        ("AI_AGENT", "claude-code_2-1-330_agent"),
        ("GIT_EDITOR", "true"),
        ("COREPACK_ENABLE_AUTO_PIN", "0"),
        ("NoDefaultCurrentDirectoryInExePath", "1"),
    ]);

    for name in [
        "AI_AGENT",
        "GIT_EDITOR",
        "COREPACK_ENABLE_AUTO_PIN",
        "NoDefaultCurrentDirectoryInExePath",
    ] {
        assert_eq!(value_of(&environment, name), None, "{name}");
    }
}

// Audit I5: outside Claude Code those are the user's own settings
#[test]
fn an_editor_the_user_chose_outside_claude_code_is_kept() {
    let environment =
        claude_environment_after(&[("GIT_EDITOR", "vim"), ("COREPACK_ENABLE_AUTO_PIN", "1")]);

    assert_eq!(value_of(&environment, "GIT_EDITOR").as_deref(), Some("vim"));
    assert_eq!(
        value_of(&environment, "COREPACK_ENABLE_AUTO_PIN").as_deref(),
        Some("1")
    );
}

// Audit I5: cmux loads a script of its own into every Node process and puts
// a folder of wrappers on the PATH
#[test]
fn what_cmux_injects_into_node_and_the_path_does_not_reach_the_session() {
    let environment = claude_environment_after(&[
        (
            "NODE_OPTIONS",
            "--require=/var/x/T/cmux-claude-node-options/restore-node-options.cjs",
        ),
        (
            "PATH",
            "/var/x/T/cmux-cli-shims/E7CE:/Applications/cmux.app/Contents/Resources/bin:\
             /usr/bin:/bin:/var/x/T//cmux-cli-shims/E7CE",
        ),
    ]);

    assert_eq!(value_of(&environment, "NODE_OPTIONS"), None);
    assert_eq!(
        value_of(&environment, "PATH").as_deref(),
        Some("/usr/bin:/bin")
    );
}

// Audit I5: NODE_OPTIONS the user set stays
#[test]
fn node_options_of_the_user_are_kept() {
    let environment = claude_environment_after(&[("NODE_OPTIONS", "--max-old-space-size=4096")]);

    assert_eq!(
        value_of(&environment, "NODE_OPTIONS").as_deref(),
        Some("--max-old-space-size=4096")
    );
}
