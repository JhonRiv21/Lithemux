fn fake(name: &str) -> String {
    format!("{}/tests/fakes/{name}", env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn doctor_reports_the_shell() {
    let shell = std::env::var("SHELL").unwrap_or_default();
    let mut command = std::process::Command::new(fake("fake-claude"));
    assert!(command.env_clear().env("SHELL", shell).status().is_ok());
}
