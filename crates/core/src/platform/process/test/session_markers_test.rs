use std::ffi::OsString;

use crate::platform::process::session_markers::{
    command_without_session_markers, without_session_markers,
};

fn cleaned(inherited: &[(&str, &str)]) -> Vec<(String, String)> {
    without_session_markers(
        inherited
            .iter()
            .map(|(name, value)| (OsString::from(name), OsString::from(value))),
    )
    .into_iter()
    .map(|(name, value)| {
        (
            name.to_string_lossy().to_string(),
            value.to_string_lossy().to_string(),
        )
    })
    .collect()
}

fn names(inherited: &[&str]) -> Vec<String> {
    let inherited: Vec<(&str, &str)> = inherited.iter().map(|name| (*name, "x")).collect();
    cleaned(&inherited)
        .into_iter()
        .map(|(name, _)| name)
        .collect()
}

// PLAN §2.0 point 19: the minimum the plan names
#[test]
fn the_markers_of_claude_code_and_cmux_are_dropped() {
    let kept = names(&[
        "CLAUDECODE",
        "CLAUDE_CODE_CHILD_SESSION",
        "CLAUDE_CODE_ENTRYPOINT",
        "CLAUDE_CODE_SSE_PORT",
        "CMUX_SURFACE_ID",
        "CMUX_SOCKET_PATH",
        "CMUX_ANYTHING_NEW",
        "HOME",
    ]);
    assert_eq!(kept, ["HOME"]);
}

// code.claude.com/docs/en/env-vars: "set by Claude Code" in what it spawns;
// audit I5: AI_AGENT
#[test]
fn what_the_documentation_says_claude_code_sets_is_dropped() {
    let kept = names(&[
        "CLAUDE_CODE_SESSION_ID",
        "CLAUDE_CODE_BRIDGE_SESSION_ID",
        "CLAUDE_CODE_MESSAGING_SOCKET",
        "CLAUDE_CODE_MESSAGING_TOKEN",
        "CLAUDE_PID",
        "CLAUDE_EFFORT",
        "CLAUDE_JOB_DIR",
        "AI_AGENT",
    ]);
    assert!(kept.is_empty(), "{kept:?}");
}

// PLAN §2.0 point 19: "no se quitan las variables de configuración"
#[test]
fn what_the_user_configures_is_kept() {
    let configured = [
        "CLAUDE_CODE_MAX_OUTPUT_TOKENS",
        "CLAUDE_CONFIG_DIR",
        "CLAUDE_CODE_FORCE_SESSION_PERSISTENCE",
        "CLAUDE_CODE_SUBAGENT_MODEL",
        "CLAUDE_ENV_FILE",
        "ANTHROPIC_API_KEY",
        "HOME",
        "LITHEMUX_HOME",
        "MYCMUX_THING",
    ];
    assert_eq!(names(&configured), configured);
}

// Audit I5: Claude Code overrides these for its tools; elsewhere they are
// the user's
#[test]
fn settings_overridden_by_claude_code_are_dropped_only_inside_it() {
    let overridden = [
        ("GIT_EDITOR", "true"),
        ("COREPACK_ENABLE_AUTO_PIN", "0"),
        ("NoDefaultCurrentDirectoryInExePath", "1"),
    ];
    let mut inside = overridden.to_vec();
    inside.push(("CLAUDECODE", "1"));

    assert!(cleaned(&inside).is_empty());
    assert_eq!(cleaned(&overridden).len(), 3);
}

// Audit I5: the script cmux loads into every Node process
#[test]
fn node_options_go_away_only_when_they_are_the_ones_cmux_set() {
    let from_cmux = "--require=/var/T/cmux-claude-node-options/restore-node-options.cjs";
    assert!(cleaned(&[("NODE_OPTIONS", from_cmux)]).is_empty());

    let from_the_user = [("NODE_OPTIONS", "--max-old-space-size=4096")];
    assert_eq!(cleaned(&from_the_user).len(), 1);
}

// Audit I5: the folder of wrappers cmux puts on the PATH
#[test]
fn the_wrappers_of_cmux_leave_the_path_and_the_rest_keeps_its_order() {
    let path =
        "/var/T/cmux-cli-shims/E7:/Users/x/.local/bin:/usr/bin:/var/T//cmux-cli-shims/E7:/bin";
    assert_eq!(
        cleaned(&[("PATH", path)]),
        [(
            "PATH".to_string(),
            "/Users/x/.local/bin:/usr/bin:/bin".to_string()
        )]
    );
    // A folder that merely resembles the name is not cmux's.
    let lookalike = "/opt/my-cmux-cli-shims-backup/bin:/usr/bin";
    assert_eq!(cleaned(&[("PATH", lookalike)])[0].1, lookalike);
}

// The folder of the cmux bundle carries an `open` that shadows the one of
// macOS; it is told by its path, not by what it contains
#[test]
fn the_bin_folder_of_the_cmux_bundle_leaves_the_path() {
    let path = "/Applications/cmux.app/Contents/Resources/bin:/usr/bin:\
                /Users/x/Applications/cmux.app/Contents/Resources/bin/:/bin";
    assert_eq!(cleaned(&[("PATH", path)])[0].1, "/usr/bin:/bin");
}

#[test]
fn folders_that_only_resemble_the_one_of_the_cmux_bundle_stay() {
    let path = "/Applications/notcmux.app/Contents/Resources/bin:\
                /Applications/cmux.app/Contents/Resources/bin/extra:\
                /Applications/cmux.app/Contents/Resources:/opt/cmux/bin";
    assert_eq!(cleaned(&[("PATH", path)])[0].1, path);
}

// Audit of the delta (point 7): only what cmux added goes; what the user
// had in NODE_OPTIONS stays, in its order
#[test]
fn only_the_script_of_cmux_leaves_node_options() {
    let script = "/var/T/cmux-claude-node-options/restore-node-options.cjs";
    for from_cmux in [
        format!("--require={script}"),
        format!("--require {script}"),
        format!("-r {script}"),
    ] {
        let options = format!("--max-old-space-size=4096 {from_cmux} --enable-source-maps");
        assert_eq!(
            cleaned(&[("NODE_OPTIONS", &options)]),
            [(
                "NODE_OPTIONS".to_string(),
                "--max-old-space-size=4096 --enable-source-maps".to_string()
            )],
            "{from_cmux}"
        );
    }
}

// Audit of the delta (point 7): another `--require` of the user is not cmux's
#[test]
fn a_require_of_the_user_stays_in_node_options() {
    let options = "--require /Users/x/tracing.cjs -r ./hook.cjs";
    assert_eq!(cleaned(&[("NODE_OPTIONS", options)])[0].1, options);
}

// Audit of the delta (point 3): what was cleaned cannot be put back one
// variable at a time, while everything else can be set
#[test]
fn a_clean_command_does_not_take_a_session_marker() {
    let mut command = command_without_session_markers("/fake/program");
    command
        .env("CLAUDECODE", "1")
        .env("CLAUDE_CODE_CHILD_SESSION", "1")
        .env("CMUX_SURFACE_ID", "abc")
        .env("LITHEMUX_SESSION_ID", "session-1");

    let set: Vec<(String, Option<String>)> = command
        .explicit_env()
        .into_iter()
        .filter(|(name, _)| name.starts_with("C") || name.starts_with("LITHEMUX"))
        .collect();

    assert!(
        set.iter()
            .all(|(name, _)| !name.starts_with("CLAUDE") && !name.starts_with("CMUX_")),
        "{set:?}"
    );
    assert!(set.contains(&(
        "LITHEMUX_SESSION_ID".to_string(),
        Some("session-1".to_string())
    )));
}
