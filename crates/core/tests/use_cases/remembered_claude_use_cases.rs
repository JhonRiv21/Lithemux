use std::path::PathBuf;

use lithemux_core::modules::launch::{ShellMode, resolve_claude};
use lithemux_core::modules::settings::{
    SettingKey, SettingsError, SettingsRepository, read_setting, write_setting,
};

use crate::support::fake_shell_probe::FakeShellProbe;
use crate::support::in_memory_store::InMemoryStore;

const LOCATE: &str = "command -v claude";
const NATIVE: &str = "/Users/x/.local/bin/claude";
const CASK: &str = "/opt/homebrew/bin/claude";
const SHIM: &str = "/var/folders/7z/abc/T/cmux-cli-shims/E7CE/claude";

// PLAN §2.0 (regression of bug 1): the claude found by the interactive
// shell is remembered for the day that shell does not answer
#[test]
fn the_claude_found_by_the_interactive_shell_is_remembered() {
    let store = InMemoryStore::new();
    let probe = FakeShellProbe::new().executable(NATIVE).answers(
        ShellMode::LoginInteractive,
        LOCATE,
        NATIVE,
    );

    resolve_claude(&store, &probe).unwrap();

    assert_eq!(
        read_setting(&store, SettingKey::LastClaudePath).unwrap(),
        Some(NATIVE.to_string())
    );
}

// PLAN §2.0 (regression of bug 1): a shell that does not answer must not
// send the user back to an older install without a word
#[test]
fn when_the_interactive_shell_fails_the_last_good_claude_is_used_with_a_warning() {
    let store = InMemoryStore::new();
    write_setting(&store, SettingKey::LastClaudePath, NATIVE).unwrap();
    let probe = FakeShellProbe::new()
        .executable(NATIVE)
        .executable(CASK)
        .answers(ShellMode::Login, LOCATE, CASK);

    let resolved = resolve_claude(&store, &probe).unwrap();

    assert_eq!(resolved.path, PathBuf::from(NATIVE));
    let warning = resolved.warning.unwrap();
    assert!(warning.contains(NATIVE), "{warning}");
    let asked: Vec<ShellMode> = probe.calls.borrow().iter().map(|call| call.0).collect();
    assert_eq!(asked, [ShellMode::LoginInteractive]);
}

// PLAN §0.3 order, step 3: a remembered claude that is gone is not used
#[test]
fn a_remembered_claude_that_no_longer_exists_gives_way_to_the_login_shell() {
    let store = InMemoryStore::new();
    write_setting(&store, SettingKey::LastClaudePath, "/gone/claude").unwrap();
    let probe = FakeShellProbe::new()
        .executable(CASK)
        .answers(ShellMode::Login, LOCATE, CASK);

    let resolved = resolve_claude(&store, &probe).unwrap();

    assert_eq!(resolved.path, PathBuf::from(CASK));
    assert!(resolved.warning.unwrap().contains("fell back"));
}

// PLAN §0.3: an injected wrapper is not used even if it was once remembered
#[test]
fn a_remembered_wrapper_is_not_used() {
    let store = InMemoryStore::new();
    write_setting(&store, SettingKey::LastClaudePath, SHIM).unwrap();
    let probe = FakeShellProbe::new()
        .executable(SHIM)
        .executable(CASK)
        .temp_dir("/var/folders/7z/abc/T")
        .answers(ShellMode::Login, LOCATE, CASK);

    assert_eq!(
        resolve_claude(&store, &probe).unwrap().path,
        PathBuf::from(CASK)
    );
}

// PLAN §0.3 order, step 1: a fixed claude_path is the user's choice and is
// not what the lookup remembers
#[test]
fn a_fixed_claude_path_does_not_change_what_is_remembered() {
    let store = InMemoryStore::new();
    write_setting(&store, SettingKey::ClaudePath, "/custom/claude").unwrap();
    let probe = FakeShellProbe::new().executable("/custom/claude");

    resolve_claude(&store, &probe).unwrap();

    assert_eq!(
        read_setting(&store, SettingKey::LastClaudePath).unwrap(),
        None
    );
}

const NOT_CLAUDE: &str = "/bin/ls";

// Audit of the delta (D-28): only a file called `claude` is an answer
#[test]
fn a_path_that_is_not_called_claude_is_never_chosen_nor_remembered() {
    let store = InMemoryStore::new();
    let probe = FakeShellProbe::new()
        .executable(NATIVE)
        .executable(NOT_CLAUDE)
        .answers(
            ShellMode::LoginInteractive,
            LOCATE,
            &format!("{NATIVE}\n{NOT_CLAUDE}\n"),
        );

    let resolved = resolve_claude(&store, &probe).unwrap();

    assert_eq!(resolved.path, PathBuf::from(NATIVE));
    assert_eq!(
        read_setting(&store, SettingKey::LastClaudePath).unwrap(),
        Some(NATIVE.to_string())
    );
}

// Audit of the delta (D-28): a poisoned memory is not used
#[test]
fn a_remembered_path_that_is_not_called_claude_is_not_used() {
    let store = InMemoryStore::new();
    write_setting(&store, SettingKey::LastClaudePath, NOT_CLAUDE).unwrap();
    let probe = FakeShellProbe::new()
        .executable(NOT_CLAUDE)
        .executable(CASK)
        .answers(ShellMode::Login, LOCATE, CASK);

    assert_eq!(
        resolve_claude(&store, &probe).unwrap().path,
        PathBuf::from(CASK)
    );
}

// Audit of the delta (point 8): the memory is a convenience; failing to
// write it must not cost the launch
#[test]
fn a_memory_that_cannot_be_written_does_not_stop_the_lookup() {
    let store = ReadOnlySettings(InMemoryStore::new());
    let probe = FakeShellProbe::new().executable(NATIVE).answers(
        ShellMode::LoginInteractive,
        LOCATE,
        NATIVE,
    );

    assert_eq!(
        resolve_claude(&store, &probe).unwrap().path,
        PathBuf::from(NATIVE)
    );
}

/// Settings that can be read and never written, like a database that is busy.
struct ReadOnlySettings(InMemoryStore);

impl SettingsRepository for ReadOnlySettings {
    fn get(&self, key: &str) -> Result<Option<String>, SettingsError> {
        self.0.get(key)
    }

    fn set(&self, key: &str, _: &str) -> Result<(), SettingsError> {
        Err(SettingsError::UnknownKey {
            key: key.to_string(),
            valid: "nothing: the store is busy".to_string(),
        })
    }

    fn remove(&self, key: &str) -> Result<(), SettingsError> {
        self.0.remove(key)
    }
}
