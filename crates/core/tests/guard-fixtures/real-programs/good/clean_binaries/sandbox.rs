pub fn cli() -> assert_cmd::Command {
    let mut command = assert_cmd::Command::cargo_bin("lithemux").unwrap();
    command.env_clear().env("PATH", "/usr/bin:/bin");
    command
}

pub fn app() -> std::process::Command {
    let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_lithemux-app"));
    command.env_clear();
    command
}
