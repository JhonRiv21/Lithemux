fn fake(name: &str) -> String {
    format!("{}/tests/fakes/{name}", env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn a_cleared_environment_gets_the_path_of_whoever_compiled() {
    let status = std::process::Command::new(fake("fake-claude"))
        .env_clear()
        .env("PATH", env!("PATH"))
        .status();
    assert!(status.is_ok());
}
