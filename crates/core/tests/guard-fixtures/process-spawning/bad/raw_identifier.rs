pub fn run(path: &str) {
    let _ = std::process::r#Command::new(path).status();
}
