use crate::sandbox::app;

// PLAN §1.2 test 1: the shipped binary must be able to turn the terminal
// font into pixels; a prompt nobody can read is not a terminal.
#[test]
fn the_app_binary_can_rasterize_the_terminal_font() {
    let output = app().arg("--check-text").output().unwrap();

    assert!(
        output.status.success(),
        "stdout: {} stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
