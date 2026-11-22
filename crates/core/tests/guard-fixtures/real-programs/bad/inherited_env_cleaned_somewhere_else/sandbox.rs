pub fn cli() -> assert_cmd::Command {
    assert_cmd::Command::cargo_bin("lithemux").unwrap()
}

pub fn something_else(program: &str) -> std::process::Command {
    let mut command = std::process::Command::new(program);
    command.env_clear();
    command
}
