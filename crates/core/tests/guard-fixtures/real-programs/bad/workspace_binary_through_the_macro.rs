#[test]
fn status_through_the_macro() {
    let status = std::process::Command::new(assert_cmd::cargo::cargo_bin!("lithemux"))
        .env_clear()
        .arg("status")
        .status();
    assert!(status.is_ok());
}
