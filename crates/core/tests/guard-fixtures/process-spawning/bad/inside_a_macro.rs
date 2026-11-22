pub fn run(path: &str) {
    let _ = dbg!(std::process::Command::new(path)).status();
}
