#[test]
fn the_binary_found_next_to_the_test() {
    let binary = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("../lithemux");
    assert!(std::process::Command::new(binary).env_clear().status().is_ok());
}
