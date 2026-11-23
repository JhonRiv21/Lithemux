use std::fs;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use lithemux_core::modules::launch::CLAUDE_PROGRAM;

/// The app binary with a clean environment.
pub fn app() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_lithemux-app"));
    command.env_clear();
    command
}

/// The fake `claude`, started the way cmux or another terminal would start
/// it, outside lithemux and with its files in `dir`. It stays until killed.
pub fn fake_claude_elsewhere(fake: &Path, dir: &Path, args: &[&str]) -> Child {
    fs::write(dir.join("fake-claude.sleep"), "60").unwrap();
    // Under the name of the real one, as cmux starts it: only a command line
    // of `claude` counts as running a conversation.
    let as_claude = dir.join(CLAUDE_PROGRAM);
    std::os::unix::fs::symlink(fake, &as_claude).unwrap();
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
