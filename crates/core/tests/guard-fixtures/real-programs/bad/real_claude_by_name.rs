#[test]
fn claude_answers_its_version() {
    let output = std::process::Command::new("claude").arg("--version").output();
    assert!(output.is_ok());
}
