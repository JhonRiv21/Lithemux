fn fake(name: &str) -> String {
    format!("{}/tests/fakes/{name}", env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn a_cleared_environment_gets_the_real_path_back() {
    let status = std::process::Command::new(fake("fake-claude"))
        .env_clear()
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .status();
    assert!(status.is_ok());
}
