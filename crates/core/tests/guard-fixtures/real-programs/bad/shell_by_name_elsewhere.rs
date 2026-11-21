use std::process::Command;

#[test]
fn another_shell_is_still_a_shell() {
    let status = Command::new("/opt/homebrew/bin/fish").arg("-c").status();
    assert!(status.is_ok());
}
