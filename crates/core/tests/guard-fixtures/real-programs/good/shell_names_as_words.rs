use std::process::Command;

fn fake(name: &str) -> String {
    format!("{}/tests/fakes/{name}", env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn the_fake_shell_is_not_a_real_one() {
    // Words in a sentence, test doubles and made-up paths are all fine.
    let message = "the zsh profile is never read";
    let pretend = "/fake/zsh";
    let status = Command::new(fake("fake-shell"))
        .env("LITHEMUX_NOTE", message)
        .arg(pretend)
        .arg("--flag=sh")
        .status();
    assert!(status.is_ok());
}
