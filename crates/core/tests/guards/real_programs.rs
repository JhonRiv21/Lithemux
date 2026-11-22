//! No test may run a real shell or the real `claude`. Text based, so it also
//! runs on fixtures that are never compiled.

use std::fs;
use std::path::{Path, PathBuf};

use proc_macro2::Delimiter;

use super::tokens::{Kind, Token, tokens_of};

const REAL_SHELLS: [&str; 3] = ["/bin/sh", "/bin/zsh", "/bin/bash"];

/// Looked up through the PATH, any of these is the machine's real shell.
const SHELL_NAMES: [&str; 4] = ["zsh", "bash", "sh", "fish"];

/// A test file that mentions any of these can start a process.
const PROCESS_MARKERS: [&str; 10] = [
    "Command",
    "ShellCommand",
    "TerminalSession",
    "SessionHost",
    "HostConfig",
    "cargo_bin",
    "LoginShellProbe",
    "TerminalSessionRunner",
    ".spawn(",
    ".output(",
];

/// The one file of each crate's tests that may build the workspace's
/// binaries, always with a clean environment.
const SANDBOX_FILE: &str = "sandbox.rs";
/// `cargo_bin`, `cargo_bin!`, `cargo_bin_cmd!`, `Command::cargo_bin`.
const BINARY_LOOKUP_PREFIX: &str = "cargo_bin";
/// `env!("CARGO_BIN_EXE_<name>")`.
const BINARY_ENV_PREFIX: &str = "CARGO_BIN_EXE_";
/// A binary taken straight from the build folder.
const BUILD_FOLDERS: [&str; 2] = ["target/debug", "target/release"];
const CLEAN_ENVIRONMENT: &str = "env_clear";
/// Variables a test may read: lithemux's own and the ones cargo sets.
const HARMLESS_PREFIXES: [&str; 2] = ["LITHEMUX_", "CARGO_"];
/// `std::env::current_exe`: the way to a binary sitting next to the test's.
const OWN_BINARY: &str = "current_exe";
/// `HostConfig::from_environment`: the app's host over the real home, the
/// real shell and, through it, the real `claude`.
const REAL_MACHINE_CONFIG: &str = "from_environment";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    pub rule: &'static str,
    pub file: String,
    pub line: usize,
}

/// Every `.rs` file of test code under `root`: `tests/` folders and the
/// `test/` folders inside `src/`.
pub fn test_sources(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect(root, &mut files);
    files.retain(|path| {
        let text = path.to_string_lossy().replace('\\', "/");
        text.contains("/tests/") || text.contains("/test/")
    });
    files.sort();
    files
}

fn collect(dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if path.is_dir() {
            // `guards` is this checker; the fixture folders are examples.
            if !matches!(
                name.as_str(),
                "target" | "arch-fixtures" | "guard-fixtures" | "guards" | "arch" | "snapshots"
            ) {
                collect(&path, files);
            }
        } else if name.ends_with(".rs") {
            files.push(path);
        }
    }
}

pub fn check_files(root: &Path, files: &[PathBuf]) -> Vec<Violation> {
    let mut violations = Vec::new();
    for path in files {
        let Ok(text) = fs::read_to_string(path) else {
            continue;
        };
        let file = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");
        check_text(&file, &text, &mut violations);
    }
    violations
}

fn check_text(file: &str, text: &str, out: &mut Vec<Violation>) {
    check_tokens(file, text, out);
    let can_start_processes = PROCESS_MARKERS.iter().any(|marker| text.contains(marker));
    for (index, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.starts_with("//") {
            continue;
        }
        let mut report = |rule: &'static str| {
            out.push(Violation {
                rule,
                file: file.to_string(),
                line: index + 1,
            });
        };
        // A real shell rebuilds the PATH from the machine's profile and finds
        // the real `claude`, so it is banned even as plain test data.
        let real_shell = REAL_SHELLS.iter().any(|shell| line.contains(shell));
        if real_shell {
            report("real-shell");
        }
        // Where processes can start, a shell named without its path (or run
        // through `env`) is still the real one.
        if can_start_processes && !real_shell && string_literals(line).any(names_a_shell) {
            report("shell-by-name");
        }
        if line.contains("\"SHELL\"") && !line.contains("fake") {
            report("shell-env");
        }
        // Where processes can start, the only `claude` is tests/fakes/fake-claude.
        if can_start_processes && (line.contains("/claude\"") || line.contains("\"claude\"")) {
            report("real-claude");
        }
    }
}

/// Rules that read the code as tokens, so a macro, an alias or a line break
/// cannot get around them.
fn check_tokens(file: &str, text: &str, out: &mut Vec<Violation>) {
    let mut report = |rule: &'static str, line: usize| {
        out.push(Violation {
            rule,
            file: file.to_string(),
            line,
        });
    };
    let Some(tokens) = tokens_of(text) else {
        return report("workspace-binary", 1);
    };
    let in_sandbox = file.rsplit('/').next() == Some(SANDBOX_FILE);
    let mut used_cleanings = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if gets_a_workspace_binary(token) {
            if !in_sandbox {
                report("workspace-binary", token.line);
            } else if !cleaned_in_the_same_block(&tokens, index, &mut used_cleanings) {
                report("inherited-env", token.line);
            }
        }
        let from_the_build = |text: &str| BUILD_FOLDERS.iter().any(|folder| text.contains(folder));
        let next_to_this_binary = token.ident() == Some(OWN_BINARY) && !in_sandbox;
        if token.text().is_some_and(from_the_build) || next_to_this_binary {
            report("workspace-binary", token.line);
        }
        if reads_the_machine(&tokens, index) {
            report("machine-env", token.line);
        }
    }
}

fn gets_a_workspace_binary(token: &Token) -> bool {
    token
        .ident()
        .is_some_and(|name| name.starts_with(BINARY_LOOKUP_PREFIX))
        || token
            .text()
            .is_some_and(|text| text.contains(BINARY_ENV_PREFIX))
}

/// Whether an `env_clear` follows the binary at `index` inside the block
/// (the function body) that builds it. Each one cleans a single binary.
fn cleaned_in_the_same_block(tokens: &[Token], index: usize, used: &mut Vec<usize>) -> bool {
    let mut depth = 0_usize;
    for (offset, token) in tokens[index + 1..].iter().enumerate() {
        let at = index + 1 + offset;
        match token.kind {
            Kind::Open(Delimiter::Brace) => depth += 1,
            Kind::Close(Delimiter::Brace) if depth == 0 => return false,
            Kind::Close(Delimiter::Brace) => depth -= 1,
            _ => {}
        }
        if token.ident() == Some(CLEAN_ENVIRONMENT) && !used.contains(&at) {
            used.push(at);
            return true;
        }
    }
    false
}

/// The environment of the machine read into a test, which is how a cleared
/// one gets the machine back: `vars()`, `var(…)` of anything but lithemux's
/// own or cargo's variables, and `env!`/`option_env!` of the same, which
/// freeze the machine that compiled the test.
fn reads_the_machine(tokens: &[Token], index: usize) -> bool {
    let after = |offset: usize| tokens.get(index + offset);
    let opens_call = |offset: usize| {
        after(offset).is_some_and(|token| token.kind == Kind::Open(Delimiter::Parenthesis))
    };
    // `SHELL` has a rule of its own.
    let harmless = |name: &str| {
        name == "SHELL"
            || HARMLESS_PREFIXES
                .iter()
                .any(|prefix| name.starts_with(prefix))
    };
    let only_argument_is_harmless = |open: usize| {
        after(open + 1).and_then(Token::text).is_some_and(harmless)
            && after(open + 2)
                .is_some_and(|token| token.kind == Kind::Close(Delimiter::Parenthesis))
    };
    match tokens[index].ident() {
        Some(REAL_MACHINE_CONFIG) => true,
        Some("vars" | "vars_os") => {
            opens_call(1)
                && after(2).is_some_and(|token| token.kind == Kind::Close(Delimiter::Parenthesis))
        }
        Some("var" | "var_os") => opens_call(1) && !only_argument_is_harmless(1),
        // The macro form: `env ! ( "NAME" )`.
        Some("env" | "option_env") => {
            after(1).is_some_and(|token| token.kind == Kind::Other)
                && opens_call(2)
                && !only_argument_is_harmless(2)
        }
        _ => false,
    }
}

/// The contents of the string literals on a line, escapes left as written.
fn string_literals(line: &str) -> impl Iterator<Item = &str> {
    let mut literals = Vec::new();
    let mut start = None;
    let mut escaped = false;
    for (index, c) in line.char_indices() {
        match (start, c) {
            (Some(_), _) if escaped => escaped = false,
            (Some(_), '\\') => escaped = true,
            (Some(from), '"') => {
                literals.push(&line[from..index]);
                start = None;
            }
            (None, '"') => start = Some(index + 1),
            _ => {}
        }
    }
    literals.into_iter()
}

/// `zsh`, `bash -l`, `/opt/homebrew/bin/fish`, `/usr/bin/env bash`: a program
/// that is a shell. Anything with `fake` in its path is a test double.
fn names_a_shell(literal: &str) -> bool {
    let is_shell = |word: &str| {
        let name = word.rsplit('/').next().unwrap_or(word);
        SHELL_NAMES.contains(&name)
    };
    let mut words = literal.split_whitespace();
    let Some(program) = words.next() else {
        return false;
    };
    if program.contains("fake") {
        return false;
    }
    if is_shell(program) {
        return true;
    }
    program.rsplit('/').next() == Some("env") && words.any(is_shell)
}
