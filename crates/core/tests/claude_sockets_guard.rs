//! PLAN principle 4 and §3.1 (22-nov): lithemux talks to Claude Code only
//! through the documented `-n` flag and the context it prints. The sockets
//! Claude Code keeps for messaging between sessions are not lithemux's to
//! read or write, from any file.

use std::fs;
use std::path::{Path, PathBuf};

/// The directory and the names Claude Code's sockets would be reached by.
const CLAUDE_SOCKETS: [&str; 3] = ["cc-socks", "cc_socks", "ccsocks"];

fn sockets_in(text: &str) -> Vec<&'static str> {
    CLAUDE_SOCKETS
        .into_iter()
        .filter(|needle| {
            text.lines().any(|line| {
                !line.trim_start().starts_with("//") && line.to_lowercase().contains(needle)
            })
        })
        .collect()
}

fn rust_sources(dir: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_sources(&path, files);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
}

// The guard itself: it must see what it is supposed to forbid, in the
// forms someone would write without meaning to dodge it.
#[test]
fn the_guard_recognizes_a_path_to_claude_sockets() {
    assert_eq!(
        sockets_in(r#"let dir = Path::new("/tmp/cc-socks");"#),
        ["cc-socks"]
    );
    assert_eq!(
        sockets_in(r#"temp.join("CC-SOCKS").join(pid)"#),
        ["cc-socks"]
    );
    assert_eq!(sockets_in("const CC_SOCKS: &str = \"x\";"), ["cc_socks"]);
    assert!(sockets_in("// the sessions talk through /tmp/cc-socks, not us").is_empty());
    assert!(sockets_in("let socket = home.app_socket();").is_empty());
}

#[test]
fn no_source_reaches_the_sockets_of_claude_code() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut files = Vec::new();
    for crate_dir in fs::read_dir(&crates).into_iter().flatten().flatten() {
        rust_sources(&crate_dir.path().join("src"), &mut files);
    }
    assert!(files.len() > 50, "found only {} files", files.len());
    let offenders: Vec<String> = files
        .iter()
        .filter_map(|file| {
            let found = sockets_in(&fs::read_to_string(file).ok()?);
            (!found.is_empty()).then(|| format!("{}: {found:?}", file.display()))
        })
        .collect();
    assert!(offenders.is_empty(), "{}", offenders.join("\n"));
}
