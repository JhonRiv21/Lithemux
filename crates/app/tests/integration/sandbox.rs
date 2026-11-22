use std::process::Command;

/// The app binary with a clean environment.
pub fn app() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_lithemux-app"));
    command.env_clear();
    command
}
