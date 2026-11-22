pub struct ShellCommand;
pub enum CliCommand {
    Open,
}

pub fn describe(command: &ShellCommand) -> &'static str {
    let _ = command;
    "Command::new is spelled here only inside a string"
}
