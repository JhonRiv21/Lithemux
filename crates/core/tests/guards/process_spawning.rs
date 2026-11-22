//! Production code builds processes in three files only, and those apply the
//! environment filter of `session_markers` (findings/H-001, D-22). Anywhere
//! else the two types that can start a process may not even be named: not
//! imported, not aliased, not written as a path.

use super::tokens::tokens_of;

/// `std::process::Command` and `portable_pty::CommandBuilder`. Nothing else
/// in the workspace's production code may carry these names.
const PROCESS_BUILDERS: [&str; 2] = ["Command", "CommandBuilder"];

/// Relative to `crates/`.
pub const ALLOWED_FILES: [&str; 3] = [
    // Builds every process of `core`, with the filter applied.
    "core/src/platform/process/session_markers.rs",
    // Receives a command to detach it; builds none.
    "core/src/platform/process/detached.rs",
    // The process of each terminal of the app, with the filter applied.
    "app/src/terminal_view/pty.rs",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mention {
    pub file: String,
    pub line: usize,
    pub name: String,
}

/// `source` is any file under a `src/` folder, Rust or not: code can be
/// pulled in from a file with another extension.
pub fn mentions_of_a_process_builder(file: &str, source: &str) -> Vec<Mention> {
    let mention = |line, name: &str| Mention {
        file: file.to_string(),
        line,
        name: name.to_string(),
    };
    let Some(tokens) = tokens_of(source) else {
        // Not Rust the lexer accepts (a snapshot, a note): look for the
        // names as whole words.
        return words_of(source)
            .filter(|(_, word)| PROCESS_BUILDERS.contains(word))
            .map(|(line, word)| mention(line, word))
            .collect();
    };
    tokens
        .iter()
        .filter_map(|token| {
            let name = token.ident()?;
            PROCESS_BUILDERS
                .contains(&name)
                .then(|| mention(token.line, name))
        })
        .collect()
}

fn words_of(text: &str) -> impl Iterator<Item = (usize, &str)> {
    text.lines().enumerate().flat_map(|(index, line)| {
        line.split(|c: char| !c.is_alphanumeric() && c != '_')
            .map(move |word| (index + 1, word))
    })
}
