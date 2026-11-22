//! What a process started by lithemux must not inherit from the session
//! lithemux itself may be running in. A `claude` that sees
//! `CLAUDE_CODE_CHILD_SESSION` believes it is nested and does not save its
//! transcript; a shell that inherits `GIT_EDITOR=true` cannot write a commit
//! message.
//!
//! This is the only list (docs/decisiones.md D-22). Variables the user sets
//! on purpose to configure Claude Code (`CLAUDE_CONFIG_DIR`,
//! `CLAUDE_CODE_MAX_OUTPUT_TOKENS`, ...) are not in it and stay untouched.

use std::ffi::{OsStr, OsString};
use std::io;
use std::path::Path;
use std::process::{Child, Command, Output, Stdio};

/// Set by Claude Code in what it spawns. The first group is documented
/// (code.claude.com/docs/en/env-vars); the second was seen in a real session
/// of v2.1.330 and is not documented.
const CLAUDE_CODE_MARKERS: [&str; 15] = [
    "CLAUDECODE",
    "CLAUDE_CODE_CHILD_SESSION",
    "CLAUDE_CODE_SESSION_ID",
    "CLAUDE_CODE_BRIDGE_SESSION_ID",
    "CLAUDE_CODE_MESSAGING_SOCKET",
    "CLAUDE_CODE_MESSAGING_TOKEN",
    "CLAUDE_PID",
    "CLAUDE_EFFORT",
    "CLAUDE_JOB_DIR",
    "CLAUDE_CODE_ENTRYPOINT",
    "CLAUDE_CODE_EXECPATH",
    "CLAUDE_CODE_SESSION_ATTENDED",
    "CLAUDE_CODE_CHILD_NOTES_DIR",
    "CLAUDE_CODE_SSE_PORT",
    "AI_AGENT",
];

/// Its presence tells that the environment comes from a Claude Code tool.
const INSIDE_CLAUDE_CODE: &str = "CLAUDECODE";

/// Ordinary settings that Claude Code overrides for its own tools. They are
/// dropped only when the environment comes from Claude Code: anywhere else
/// they are the user's.
const OVERRIDDEN_BY_CLAUDE_CODE: [&str; 3] = [
    "GIT_EDITOR",
    "COREPACK_ENABLE_AUTO_PIN",
    "NoDefaultCurrentDirectoryInExePath",
];

/// Everything cmux exports into its tabs starts with this.
const CMUX_PREFIX: &str = "CMUX_";
/// cmux adds to `NODE_OPTIONS` a script to load from a folder with this name.
const CMUX_NODE_OPTIONS: &str = "cmux-claude-node-options";
/// cmux puts a folder with this name, full of wrappers, on the `PATH`.
const CMUX_SHIMS: &str = "cmux-cli-shims";
/// And the tools inside its own bundle, among them an `open` that would
/// shadow the one of macOS.
const CMUX_BUNDLE_BIN: [&str; 4] = ["cmux.app", "Contents", "Resources", "bin"];

fn is_session_marker(name: &str) -> bool {
    CLAUDE_CODE_MARKERS.contains(&name) || name.starts_with(CMUX_PREFIX)
}

/// `inherited` as a child of lithemux should see it.
pub fn without_session_markers<I>(inherited: I) -> Vec<(OsString, OsString)>
where
    I: IntoIterator<Item = (OsString, OsString)>,
{
    let inherited: Vec<(OsString, OsString)> = inherited.into_iter().collect();
    let inside_claude_code = inherited.iter().any(|(name, _)| name == INSIDE_CLAUDE_CODE);
    inherited
        .into_iter()
        .filter_map(|(name, value)| {
            let cleaned = match name.to_str() {
                Some(name) => cleaned_value(name, &value, inside_claude_code)?,
                None => value,
            };
            Some((name, cleaned))
        })
        .collect()
}

/// `None` when the variable goes away; otherwise its value, cleaned.
fn cleaned_value(name: &str, value: &OsStr, inside_claude_code: bool) -> Option<OsString> {
    if is_session_marker(name) {
        return None;
    }
    if inside_claude_code && OVERRIDDEN_BY_CLAUDE_CODE.contains(&name) {
        return None;
    }
    let text = value.to_str();
    match name {
        "NODE_OPTIONS" => match text {
            Some(options) => node_options_without_cmux(options).map(Into::into),
            None => Some(value.to_os_string()),
        },
        "PATH" => Some(text.map_or_else(
            || value.to_os_string(),
            |path| path_without_cmux(path).into(),
        )),
        _ => Some(value.to_os_string()),
    }
}

/// The options without the script cmux makes Node load, written as
/// `--require=<path>`, `--require <path>` or `-r <path>`. `None` when
/// nothing else was there.
fn node_options_without_cmux(options: &str) -> Option<String> {
    let mut kept: Vec<&str> = Vec::new();
    for option in options.split_whitespace() {
        if option.contains(CMUX_NODE_OPTIONS) {
            // The flag that introduced this path goes with it.
            if !option.starts_with('-') && matches!(kept.last(), Some(&("--require" | "-r"))) {
                kept.pop();
            }
        } else {
            kept.push(option);
        }
    }
    (!kept.is_empty()).then(|| kept.join(" "))
}

fn path_without_cmux(path: &str) -> String {
    path.split(':')
        .filter(|entry| !belongs_to_cmux(entry))
        .collect::<Vec<_>>()
        .join(":")
}

/// A folder of cmux wrappers, or exactly the `bin` folder of a cmux bundle.
/// Both are told by their path, never by what they contain.
fn belongs_to_cmux(path_entry: &str) -> bool {
    let folders: Vec<&str> = path_entry
        .split('/')
        .filter(|folder| !folder.is_empty())
        .collect();
    folders.contains(&CMUX_SHIMS) || folders.ends_with(&CMUX_BUNDLE_BIN)
}

/// A process about to start, with this process's environment as cleaned by
/// `without_session_markers`. Every process `core` starts is built here.
///
/// It is a `std::process::Command` with less on it: there is no way to
/// replace the environment wholesale or to clear it, and `env` does not
/// take a session marker, so what was cleaned cannot come back by accident.
pub struct CleanCommand(Command);

pub fn command_without_session_markers(program: impl AsRef<OsStr>) -> CleanCommand {
    let mut command = Command::new(program);
    command
        .env_clear()
        .envs(without_session_markers(std::env::vars_os()));
    CleanCommand(command)
}

impl CleanCommand {
    pub fn arg(&mut self, arg: impl AsRef<OsStr>) -> &mut Self {
        self.0.arg(arg);
        self
    }

    pub fn args<S: AsRef<OsStr>>(&mut self, args: impl IntoIterator<Item = S>) -> &mut Self {
        self.0.args(args);
        self
    }

    /// Sets one variable. A session marker is left out without a word: the
    /// caller is passing on something it inherited.
    pub fn env(&mut self, name: impl AsRef<OsStr>, value: impl AsRef<OsStr>) -> &mut Self {
        if !name.as_ref().to_str().is_some_and(is_session_marker) {
            self.0.env(name, value);
        }
        self
    }

    pub fn env_remove(&mut self, name: impl AsRef<OsStr>) -> &mut Self {
        self.0.env_remove(name);
        self
    }

    pub fn current_dir(&mut self, directory: impl AsRef<Path>) -> &mut Self {
        self.0.current_dir(directory);
        self
    }

    pub fn stdin(&mut self, stdin: Stdio) -> &mut Self {
        self.0.stdin(stdin);
        self
    }

    pub fn stdout(&mut self, stdout: Stdio) -> &mut Self {
        self.0.stdout(stdout);
        self
    }

    pub fn stderr(&mut self, stderr: Stdio) -> &mut Self {
        self.0.stderr(stderr);
        self
    }

    pub fn spawn(&mut self) -> io::Result<Child> {
        self.0.spawn()
    }

    pub fn output(&mut self) -> io::Result<Output> {
        self.0.output()
    }

    /// For the siblings that set up how the process starts, never its
    /// environment.
    pub(super) fn as_command_mut(&mut self) -> &mut Command {
        &mut self.0
    }

    /// The variables set or removed one by one, for the tests.
    #[cfg(test)]
    pub(super) fn explicit_env(&self) -> Vec<(String, Option<String>)> {
        let text = |value: &OsStr| value.to_string_lossy().to_string();
        self.0
            .get_envs()
            .map(|(name, value)| (text(name), value.map(text)))
            .collect()
    }
}
