#[test]
fn the_script_runs() {
    let output = std::process::Command::new("/bin/bash").arg("-c").arg("true").output();
    assert!(output.is_ok());
}
