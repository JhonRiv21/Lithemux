const NAME: &str = "PATH";

fn fake(name: &str) -> String {
    format!("{}/tests/fakes/{name}", env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn a_cleared_environment_gets_a_variable_named_elsewhere() {
    let status = std::process::Command::new(fake("fake-claude"))
        .env_clear()
        .env(NAME, std::env::var(NAME).unwrap_or_default())
        .status();
    assert!(status.is_ok());
}
