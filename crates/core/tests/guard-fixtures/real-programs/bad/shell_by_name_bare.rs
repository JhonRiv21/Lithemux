use lithemux_app::terminal_view::{GridSize, ShellCommand, TerminalSession};

#[test]
fn a_pane_starts() {
    // Resolved through the PATH, this is the user's real zsh and its profile.
    let command = ShellCommand::new("zsh");
    assert!(TerminalSession::spawn(&command, GridSize::INITIAL).is_ok());
}
