use std::path::Path;

pub const HOOK_BIN_NAME: &str = "lithemux-hook";

pub struct HookBinding {
    pub claude_event: &'static str,
    pub subcommand: &'static str,
    pub timeout_secs: u32,
}

/// Seven events. `PreToolUse` is left out on purpose: it would run before
/// every tool. `PostToolUse` and `PostToolUseFailure` are in only to leave
/// `waiting` once a permission was granted.
pub const HOOK_BINDINGS: [HookBinding; 7] = [
    HookBinding {
        claude_event: "SessionStart",
        subcommand: "session-start",
        timeout_secs: 5,
    },
    HookBinding {
        claude_event: "UserPromptSubmit",
        subcommand: "prompt-submit",
        timeout_secs: 2,
    },
    HookBinding {
        claude_event: "Stop",
        subcommand: "stop",
        timeout_secs: 2,
    },
    HookBinding {
        claude_event: "Notification",
        subcommand: "notification",
        timeout_secs: 2,
    },
    HookBinding {
        claude_event: "SessionEnd",
        subcommand: "session-end",
        timeout_secs: 2,
    },
    HookBinding {
        claude_event: "PostToolUse",
        subcommand: "post-tool-use",
        timeout_secs: 2,
    },
    HookBinding {
        claude_event: "PostToolUseFailure",
        subcommand: "post-tool-use-failure",
        timeout_secs: 2,
    },
];

/// The tools of Claude Code's own messaging between sessions, by their
/// documented names: a session reports to its coordinator with them (PLAN
/// §3.1). Allowed so that a report never stops on a permission prompt; a
/// `deny` of the user's still wins over this `allow`.
pub const MESSAGING_TOOLS: [&str; 2] = ["SendMessage", "ListAgents"];

/// The `--settings` payload. It is passed on every launch because Claude Code
/// does not restore it on resume. Claude Code merges its lists with those
/// of the user's settings files rather than replacing them.
pub fn hooks_settings_json(hook_bin: &Path) -> String {
    let hook = shell_quote(&hook_bin.display().to_string());
    let mut hooks = serde_json::Map::new();
    for binding in &HOOK_BINDINGS {
        hooks.insert(
            binding.claude_event.to_string(),
            serde_json::json!([{
                "hooks": [{
                    "type": "command",
                    "command": format!("{hook} {}", binding.subcommand),
                    "timeout": binding.timeout_secs,
                }]
            }]),
        );
    }
    serde_json::json!({
        "hooks": hooks,
        "permissions": { "allow": MESSAGING_TOOLS },
    })
    .to_string()
}

// Hook commands run through `sh -c`, so a path with spaces must be quoted.
fn shell_quote(value: &str) -> String {
    let plain = value
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'/' | b'.' | b'_' | b'-' | b'+'));
    if plain && !value.is_empty() {
        value.to_string()
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}
