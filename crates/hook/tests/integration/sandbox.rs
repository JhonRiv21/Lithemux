use assert_cmd::Command;

/// The hook binary with a clean environment: nothing of the machine, and in
/// particular no session of lithemux or Claude Code, gets through.
pub fn hook() -> Command {
    let mut command = Command::cargo_bin("lithemux-hook").unwrap();
    command.env_clear();
    command
}
