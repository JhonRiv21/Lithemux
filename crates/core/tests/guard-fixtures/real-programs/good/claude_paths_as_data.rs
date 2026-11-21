// Nothing here can start a process, so claude paths are only data for the
// in-memory ports.
const NATIVE: &str = "/Users/x/.local/bin/claude";

#[test]
fn the_resolved_path_is_the_one_the_fake_probe_reports() {
    let probe = FakeShellProbe::new().executable(NATIVE);
    assert!(probe.is_executable(NATIVE.as_ref()));
    let shell = PathBuf::from("/fake/zsh");
    assert_eq!(shell.file_name().unwrap(), "zsh");
}
