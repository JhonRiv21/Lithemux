use std::process::Command;

pub fn run(paths: &[&str]) {
    for mut command in paths.iter().map(Command::new) {
        let _ = command.status();
    }
}
