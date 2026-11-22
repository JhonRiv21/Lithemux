use std::path::PathBuf;

use crate::modules::launch::domain::claude_candidates::{candidate_paths, is_called_claude};

// Audit of the delta (D-28): whatever answers for `claude` is called `claude`
#[test]
fn only_absolute_paths_to_a_file_called_claude_are_candidates() {
    let output = "welcome\n/bin/ls\n/Users/x/.local/bin/claude\nclaude\n\
                  /opt/claude/bin/node\n/opt/homebrew/bin/claude\n/Users/x/.local/bin/claude\n";
    assert_eq!(
        candidate_paths(output),
        [
            PathBuf::from("/Users/x/.local/bin/claude"),
            PathBuf::from("/opt/homebrew/bin/claude"),
        ]
    );
}

#[test]
fn a_name_that_only_contains_claude_is_not_claude() {
    assert!(is_called_claude("/a/claude".as_ref()));
    for other in [
        "/a/claude-wrapper",
        "/a/claude.sh",
        "/claude/ls",
        "/a/Claude",
    ] {
        assert!(!is_called_claude(other.as_ref()), "{other}");
    }
}
