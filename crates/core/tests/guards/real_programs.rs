//! No test may run a real shell or the real `claude`. Text based, so it also
//! runs on fixtures that are never compiled.

use std::fs;
use std::path::{Path, PathBuf};

const REAL_SHELLS: [&str; 3] = ["/bin/sh", "/bin/zsh", "/bin/bash"];

/// A test file that mentions any of these can start a process.
const PROCESS_MARKERS: [&str; 6] = [
    "Command",
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
        if REAL_SHELLS.iter().any(|shell| line.contains(shell)) {
            report("real-shell");
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
