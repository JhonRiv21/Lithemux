use crate::modules::launch::domain::conversation_in_use::conversation_in;

const ID: &str = "0b9f4f8e-9d1c-4c4f-9a59-3f1c2d7e8a10";

fn args(line: &[&str]) -> Vec<String> {
    line.iter().map(|arg| arg.to_string()).collect()
}

// PLAN §3.4: a `claude` that runs the conversation, however it was named
#[test]
fn the_conversation_is_read_from_each_way_of_naming_it() {
    for line in [
        vec!["claude", "--resume", ID],
        vec![
            "node",
            "/opt/homebrew/bin/claude",
            "-r",
            ID,
            "--model",
            "opus",
        ],
        vec![
            "claude",
            "--dangerously-skip-permissions",
            "--session-id",
            ID,
        ],
        vec!["claude", &format!("--resume={ID}")],
        vec!["claude", &format!("--session-id={ID}")],
    ] {
        assert_eq!(conversation_in(&args(&line)), Some(ID), "{line:?}");
    }
}

// PLAN §3.4: only a conversation named by its flag counts
#[test]
fn a_command_line_without_a_conversation_names_none() {
    for line in [
        vec!["claude"],
        vec!["claude", "--continue"],
        vec!["claude", "--resume"],
        vec!["grep", ID],
        vec!["claude", "--resumed", ID],
        vec!["zsh", "-c", &format!("claude --resume {ID}")],
    ] {
        assert_eq!(conversation_in(&args(&line)), None, "{line:?}");
    }
}

// ronda 3B punto 7: `-r` is any program's option; only `claude` counts,
// called by its name or run by an interpreter
#[test]
fn only_a_command_line_of_claude_names_a_conversation() {
    for line in [
        vec!["grep", "-r", ID, "."],
        vec!["rsync", "--session-id", ID],
        vec!["/opt/tools/fake-claude", "--resume", ID],
        vec!["node", "/opt/cli.js", "--resume", ID],
        vec!["python3", "claude", "--resume", ID],
    ] {
        assert_eq!(conversation_in(&args(&line)), None, "{line:?}");
    }
    for line in [
        vec!["/Users/x/.local/bin/claude", "-r", ID],
        vec![
            "/opt/homebrew/bin/node",
            "/opt/homebrew/bin/claude",
            "--resume",
            ID,
        ],
        vec!["/opt/homebrew/bin/bun", "/tmp/x/claude", "--session-id", ID],
    ] {
        assert_eq!(conversation_in(&args(&line)), Some(ID), "{line:?}");
    }
}

// ronda 3B punto 7: a flag without its value does not take the next flag
#[test]
fn a_flag_followed_by_another_flag_names_no_conversation() {
    assert_eq!(
        conversation_in(&args(&["claude", "--resume", "--fork-session", ID])),
        None
    );
    assert_eq!(
        conversation_in(&args(&["claude", "--session-id", "-r"])),
        None
    );
}
