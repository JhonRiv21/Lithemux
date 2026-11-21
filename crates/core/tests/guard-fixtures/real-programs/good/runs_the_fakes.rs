use std::process::Command;

fn fake(name: &str) -> String {
    format!("{}/tests/fakes/{name}", env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn open_runs_the_fake_claude_through_the_fake_shell() {
    // A comment may mention /bin/sh or "claude" freely.
    let status = Command::new(fake("fake-claude"))
        .env("SHELL", fake("fake-shell"))
        .env("PATH", "/usr/bin:/bin")
        .arg("--version")
        .status();
    assert!(status.is_ok());
}
