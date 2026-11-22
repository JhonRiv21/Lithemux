use lithemux_core::platform::process::session_markers::command_without_session_markers;

/// Not a `Command::new`: the filter builds it.
pub fn run(path: &str) {
    let _ = command_without_session_markers(path).status();
}
