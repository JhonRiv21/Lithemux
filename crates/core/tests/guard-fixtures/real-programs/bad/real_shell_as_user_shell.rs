#[test]
fn open_explains_when_claude_cannot_be_found() {
    let probe = LoginShellProbe::new("/bin/sh");
    assert!(probe.run(ShellMode::Login, "command -v x", TIMEOUT).is_none());
}
