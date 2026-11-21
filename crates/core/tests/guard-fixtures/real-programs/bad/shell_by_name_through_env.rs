use std::process::Command;

#[test]
fn a_login_shell_prints_the_path() {
    let output = Command::new("/usr/bin/env bash -l").output();
    assert!(output.is_ok());
}
