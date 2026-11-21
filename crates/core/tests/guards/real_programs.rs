//! No test may run a real shell or the real `claude`. Text based, so it also
//! runs on fixtures that are never compiled.

use std::fs;
use std::path::{Path, PathBuf};

const REAL_SHELLS: [&str; 3] = ["/bin/sh", "/bin/zsh", "/bin/bash"];

/// Looked up through the PATH, any of these is the machine's real shell.
const SHELL_NAMES: [&str; 4] = ["zsh", "bash", "sh", "fish"];

/// A test file that mentions any of these can start a process.
const PROCESS_MARKERS: [&str; 8] = [
    "Command",
    "ShellCommand",
    "TerminalSession",
    "cargo_bin",
    "LoginShellProbe",
    "TerminalSessionRunner",
    ".spawn(",
    ".output(",
];

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
