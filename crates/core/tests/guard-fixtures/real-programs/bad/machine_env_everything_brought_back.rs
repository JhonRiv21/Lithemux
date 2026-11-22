fn fake(name: &str) -> String {
    format!("{}/tests/fakes/{name}", env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn a_cleared_environment_gets_everything_back() {
    let status = std::process::Command::new(fake("fake-claude"))
        .env_clear()
        .envs(std::env::vars())
        .status();
    assert!(status.is_ok());
}
