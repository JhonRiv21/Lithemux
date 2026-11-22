#[test]
fn status_runs_with_whatever_the_machine_exports() {
    let mut command = assert_cmd::Command::cargo_bin("lithemux").unwrap();
    command.env_clear().arg("status").assert().success();
}
