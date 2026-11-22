fn fake(name: &str) -> String {
    format!("{}/tests/fakes/{name}", env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn the_clean_environment_of_another_command_covers_nothing() {
    let _ = std::process::Command::new(fake("fake-claude"))
        .env_clear()
        .status();
    let hook = std::process::Command::new(env!("CARGO_BIN_EXE_lithemux-hook")).status();
    assert!(hook.is_ok());
}
