use std::process::Command as Cmd;

pub fn run(path: &str) {
    let _ = Cmd::new(path).status();
}
