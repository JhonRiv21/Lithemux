use std::path::Path;

#[test]
fn the_binary_straight_from_the_build_folder() {
    let binary = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/debug/lithemux");
    assert!(std::process::Command::new(binary).status().is_ok());
}
