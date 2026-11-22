use crate::sandbox::cli;

#[test]
fn status_runs_through_the_sandbox() {
    // `cargo_bin` and "CARGO_BIN_EXE_" may be mentioned in a comment.
    cli().arg("status").assert().success();
    let variable = std::env::var("LITHEMUX_HOME");
    assert!(variable.is_err());
}
