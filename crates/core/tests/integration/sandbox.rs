use std::process::Command;

/// This test binary, to be run again as a helper process, with a clean
/// environment.
pub fn this_test_binary() -> Command {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command.env_clear();
    command
}
