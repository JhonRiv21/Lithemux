use std::path::Path;

/// The arguments with which `claude` is told which conversation to take:
/// `--resume <id>` (or `-r <id>`) and `--session-id <id>`, also written as
/// `--flag=<id>`.
pub const CONVERSATION_FLAGS: [&str; 3] = ["--resume", "-r", "--session-id"];

/// What `claude` is called on disk, by its native installer and by npm.
pub const CLAUDE_PROGRAM: &str = "claude";
/// Programs that run a script named `claude`: npm's is run by `node`, a
/// wrapper by a shell.
const INTERPRETERS: [&str; 7] = ["node", "bun", "deno", "sh", "bash", "zsh", "dash"];

fn file_name(arg: &str) -> Option<&str> {
    Path::new(arg).file_name()?.to_str()
}

/// The arguments after the program, when the command line is `claude`'s:
/// `claude …`, or `<interpreter> …/claude …`.
fn after_claude(args: &[String]) -> Option<&[String]> {
    let program = file_name(args.first()?)?;
    if program == CLAUDE_PROGRAM {
        return args.get(1..);
    }
    let script = args.get(1).and_then(|arg| file_name(arg));
    (INTERPRETERS.contains(&program) && script == Some(CLAUDE_PROGRAM)).then(|| args.get(2..))?
}

/// The conversation a command line of `claude` takes, if it names one.
/// Another program's `-r` is not one, and neither is a flag without its
/// value (`--resume --fork-session`).
///
/// Known limits (D-60): `--continue`, `--resume <name>` and a `/clear` or
/// `/resume` inside the session change the conversation without the
/// command line saying which.
pub fn conversation_in(args: &[String]) -> Option<&str> {
    let mut args = after_claude(args)?.iter();
    while let Some(arg) = args.next() {
        for flag in CONVERSATION_FLAGS {
            if arg == flag {
                return args
                    .next()
                    .map(String::as_str)
                    .filter(|value| !value.starts_with('-'));
            }
            if let Some(id) = arg
                .strip_prefix(flag)
                .and_then(|rest| rest.strip_prefix('='))
            {
                return Some(id);
            }
        }
    }
    None
}
