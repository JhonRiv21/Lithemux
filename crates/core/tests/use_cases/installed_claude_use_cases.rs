//! PLAN §3.2: the version of the `claude` a launch would use now, found the
//! way a launch finds it.

use lithemux_core::modules::launch::{ShellMode, installed_claude_version};
use lithemux_core::modules::settings::{SettingKey, write_setting};

use crate::support::fake_shell_probe::FakeShellProbe;
use crate::support::in_memory_store::InMemoryStore;

const LOCATE: &str = "command -v claude";
const NATIVE: &str = "/Users/x/.local/bin/claude";
const OLD: &str = "/usr/local/bin/claude";

#[test]
fn the_version_is_the_one_of_the_claude_a_launch_would_pick() {
    let store = InMemoryStore::new();
    let probe = FakeShellProbe::new()
        .executable(NATIVE)
        .executable(OLD)
        .answers(ShellMode::LoginInteractive, LOCATE, NATIVE)
        .version(NATIVE, "2.1.331 (Claude Code)")
        .version(OLD, "1.0.0 (Claude Code)");
    assert_eq!(
        installed_claude_version(&store, &probe).as_deref(),
        Some("2.1.331")
    );
}

// The user's `claude_path` decides, as it does for a launch.
#[test]
fn a_fixed_claude_path_is_the_one_asked() {
    let store = InMemoryStore::new();
    write_setting(&store, SettingKey::ClaudePath, OLD).unwrap();
    let probe = FakeShellProbe::new()
        .executable(NATIVE)
        .executable(OLD)
        .answers(ShellMode::LoginInteractive, LOCATE, NATIVE)
        .version(NATIVE, "2.1.331 (Claude Code)")
        .version(OLD, "1.0.0 (Claude Code)");
    assert_eq!(
        installed_claude_version(&store, &probe).as_deref(),
        Some("1.0.0")
    );
}

// No `claude`, or one that does not answer: nothing to compare, no notice.
#[test]
fn without_a_claude_that_answers_there_is_no_version() {
    let store = InMemoryStore::new();
    assert_eq!(
        installed_claude_version(&store, &FakeShellProbe::new()),
        None
    );
    let silent = FakeShellProbe::new().executable(NATIVE).answers(
        ShellMode::LoginInteractive,
        LOCATE,
        NATIVE,
    );
    assert_eq!(installed_claude_version(&store, &silent), None);
}
