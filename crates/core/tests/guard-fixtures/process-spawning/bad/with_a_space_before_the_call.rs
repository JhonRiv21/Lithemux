use std::process::Command;

pub fn run(path: &str) {
    let _ = Command::new (path).status();
}
