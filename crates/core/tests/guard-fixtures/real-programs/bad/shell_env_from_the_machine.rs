#[test]
fn doctor_reports_the_shell() {
    let shell = std::env::var("SHELL").unwrap_or_default();
    let mut command = assert_cmd::Command::cargo_bin("lithemux").unwrap();
    command.env("SHELL", shell).arg("doctor").assert().success();
}
