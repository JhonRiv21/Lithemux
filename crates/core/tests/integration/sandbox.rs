use std::fs;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use lithemux_core::modules::launch::CLAUDE_PROGRAM;

/// This test binary, to be run again as a helper process, with a clean
/// environment.
pub fn this_test_binary() -> Command {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command.env_clear();
    command
}

/// The fake `claude`, started the way another program (cmux, a terminal)
/// would start it, outside any lithemux home: it stays until killed.
/// `dir` is where it keeps its files.
pub fn fake_claude_elsewhere(dir: &Path, args: &[&str]) -> Child {
    fs::write(dir.join("fake-claude.sleep"), "60").unwrap();
    // Under the name of the real one, as cmux starts it: only a command line
    // of `claude` counts as running a conversation.
    let as_claude = dir.join(CLAUDE_PROGRAM);
    std::os::unix::fs::symlink(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fakes/fake-claude"),
        &as_claude,
    )
    .unwrap();
    let child = Command::new(&as_claude)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("LITHEMUX_HOME", dir)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    // Once it wrote its pid, all it does is sleep.
    let deadline = Instant::now() + Duration::from_secs(10);
    while !dir.join("fake-claude.pid").exists() {
        assert!(Instant::now() < deadline, "the fake claude did not start");
        std::thread::sleep(Duration::from_millis(20));
    }
    // It wrote its pid before reading its files in subshells, which show
    // its command line for a moment: once they are done, only it does.
    std::thread::sleep(Duration::from_millis(300));
    child
}
