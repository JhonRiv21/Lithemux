pub fn run(path: &str) {
    let _ = <std::process::Command>::new(path).status();
}
