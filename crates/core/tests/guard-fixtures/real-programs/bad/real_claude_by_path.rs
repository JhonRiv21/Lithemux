#[test]
fn open_launches_the_installed_claude() {
    let sandbox = Sandbox::new();
    sandbox.run(&["config", "set", "claude_path", "/opt/homebrew/bin/claude"]);
    let output = sandbox.lithemux(&["open", "demo/ejecutora"]).output();
    assert!(output.is_ok());
}
