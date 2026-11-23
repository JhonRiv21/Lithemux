use std::path::PathBuf;
use std::time::Duration;

use lithemux_core::modules::launch::{
    ClaudePathProblem, LaunchError, ShellMode, inspect_claude, resolve_claude,
};
use lithemux_core::modules::settings::{SettingKey, write_setting};

use crate::support::fake_shell_probe::FakeShellProbe;
use crate::support::in_memory_store::InMemoryStore;

const LOCATE: &str = "command -v claude";
const NATIVE: &str = "/Users/x/.local/bin/claude";
const CASK: &str = "/opt/homebrew/bin/claude";
const SHIM: &str = "/var/folders/7z/abc/T/cmux-cli-shims/E7CE/claude";

// PLAN §0.3 order, step 1
#[test]
fn claude_path_wins_and_the_shell_is_not_even_asked() {
    let store = InMemoryStore::new();
    write_setting(&store, SettingKey::ClaudePath, "/custom/claude").unwrap();
    let probe = FakeShellProbe::new()
        .executable("/custom/claude")
        .executable(NATIVE)
        .answers(ShellMode::LoginInteractive, LOCATE, NATIVE);

    let resolved = resolve_claude(&store, &probe).unwrap();

    assert_eq!(resolved.path, PathBuf::from("/custom/claude"));
    assert_eq!(resolved.warning, None);
    assert!(probe.calls.borrow().is_empty());
}

/// What `resolve_claude` refuses for a `claude_path`, with a `claude` on
/// the PATH it could have run instead.
fn refused_with(value: &str, probe: FakeShellProbe) -> (String, ClaudePathProblem) {
    let store = InMemoryStore::new();
    write_setting(&store, SettingKey::ClaudePath, value).unwrap();
    let probe = probe
        .executable(NATIVE)
        .answers(ShellMode::LoginInteractive, LOCATE, NATIVE);

    let refused = resolve_claude(&store, &probe);

    assert!(probe.calls.borrow().is_empty(), "the shell was asked");
    match refused {
        Err(LaunchError::ClaudePathUnusable { value, problem }) => (value, problem),
        other => panic!("{other:?}"),
    }
}

// PLAN §3.7, carried over from 3B: an unusable claude_path says why and
// never runs another `claude` in its place.
#[test]
fn a_claude_path_where_nothing_is_says_so_and_no_other_claude_is_looked_for() {
    let refused = refused_with("/gone/claude", FakeShellProbe::new());

    assert_eq!(
        refused,
        ("/gone/claude".to_string(), ClaudePathProblem::Missing)
    );
}

// PLAN §3.7, carried over from 3B (ronda 3C): settings are stored without
// JSON quotes, so a quoted path is not the path it quotes, even when that
// one runs.
#[test]
fn a_claude_path_stored_with_quotes_says_it_has_quotes() {
    let probe = FakeShellProbe::new()
        .executable("/custom/claude")
        .executable("\"/custom/claude\"");

    let refused = refused_with("\"/custom/claude\"", probe);

    assert_eq!(refused.1, ClaudePathProblem::Quoted);
}

// ronda 3C: a folder is not a program.
#[test]
fn a_claude_path_to_a_folder_says_it_is_a_folder() {
    let refused = refused_with(
        "/opt/claude",
        FakeShellProbe::new().directory("/opt/claude"),
    );

    assert_eq!(refused.1, ClaudePathProblem::Directory);
}

// ronda 3C: a file that cannot run.
#[test]
fn a_claude_path_to_a_file_that_cannot_run_says_so() {
    let refused = refused_with(
        "/opt/claude",
        FakeShellProbe::new().plain_file("/opt/claude"),
    );

    assert_eq!(refused.1, ClaudePathProblem::NotExecutable);
}

// PLAN §3.7, carried over from 3B: doctor tells the same, and does not
// show another `claude` as the one a launch would use.
#[test]
fn doctor_reports_an_unusable_claude_path_and_chooses_nothing() {
    let store = InMemoryStore::new();
    write_setting(&store, SettingKey::ClaudePath, "/gone/claude").unwrap();
    let probe = FakeShellProbe::new().executable(NATIVE).answers(
        ShellMode::LoginInteractive,
        LOCATE,
        NATIVE,
    );

    let inspection = inspect_claude(&store, &probe);

    assert!(matches!(
        inspection.chosen,
        Err(LaunchError::ClaudePathUnusable { .. })
    ));
    assert!(inspection.installations.iter().all(|found| !found.chosen));
}

// PLAN §0.3 order, step 2
#[test]
fn the_interactive_login_shell_is_asked_first_with_a_three_second_limit() {
    let probe = FakeShellProbe::new()
        .executable(NATIVE)
        .executable(CASK)
        .answers(ShellMode::LoginInteractive, LOCATE, NATIVE)
        .answers(ShellMode::Login, LOCATE, CASK);

    let resolved = resolve_claude(&InMemoryStore::new(), &probe).unwrap();

    assert_eq!(resolved.path, PathBuf::from(NATIVE));
    assert_eq!(resolved.warning, None);
    assert_eq!(
        *probe.calls.borrow(),
        vec![(
            ShellMode::LoginInteractive,
            LOCATE.to_string(),
            Duration::from_secs(3)
        )]
    );
}

// PLAN §0.3 order, step 2: the last line that is an absolute, executable path
#[test]
fn whatever_the_zshrc_prints_is_ignored() {
    let noisy = format!(
        "Welcome back!\n\u{1b}[32mnvm ready\u{1b}[0m\n/not/executable/claude\nclaude: aliased to x\n{NATIVE}\n"
    );
    let probe = FakeShellProbe::new().executable(NATIVE).answers(
        ShellMode::LoginInteractive,
        LOCATE,
        &noisy,
    );
    let resolved = resolve_claude(&InMemoryStore::new(), &probe).unwrap();
    assert_eq!(resolved.path, PathBuf::from(NATIVE));
}

// PLAN §0.3: candidates under temporary folders or `shims` are discarded
#[test]
fn an_injected_wrapper_is_never_chosen() {
    for wrapper in [
        SHIM,
        "/tmp/wrap/claude",
        "/private/tmp/wrap/claude",
        "/private/var/folders/aa/T/claude",
        "/custom-tmp/claude",
        "/Users/x/.tool/shims/claude",
    ] {
        let probe = FakeShellProbe::new()
            .temp_dir("/custom-tmp")
            .executable(wrapper)
            .executable(CASK)
            .answers(ShellMode::LoginInteractive, LOCATE, wrapper)
            .answers(ShellMode::Login, LOCATE, CASK);
        let resolved = resolve_claude(&InMemoryStore::new(), &probe).unwrap();
        assert_eq!(resolved.path, PathBuf::from(CASK), "{wrapper}");
    }
}

// PLAN §0.3 order, step 3
#[test]
fn a_shell_that_times_out_falls_back_to_the_login_shell_with_a_warning() {
    // No answer for the interactive mode: the probe reports a failure/timeout.
    let probe = FakeShellProbe::new()
        .executable(CASK)
        .answers(ShellMode::Login, LOCATE, CASK);

    let resolved = resolve_claude(&InMemoryStore::new(), &probe).unwrap();

    assert_eq!(resolved.path, PathBuf::from(CASK));
    assert!(resolved.warning.is_some());
    let modes: Vec<ShellMode> = probe.calls.borrow().iter().map(|call| call.0).collect();
    assert_eq!(modes, vec![ShellMode::LoginInteractive, ShellMode::Login]);
}

// PLAN §0.3
#[test]
fn no_usable_claude_anywhere_is_an_error() {
    let probe = FakeShellProbe::new()
        .answers(ShellMode::LoginInteractive, LOCATE, "claude not found\n")
        .answers(ShellMode::Login, LOCATE, SHIM);
    assert!(matches!(
        resolve_claude(&InMemoryStore::new(), &probe),
        Err(LaunchError::ClaudeNotFound(_))
    ));
}

// PLAN §0.6 doctor: every claude found, its version and the chosen one
#[test]
fn doctor_lists_every_install_and_flags_different_versions() {
    let listing = format!("{NATIVE}\n{CASK}\n{CASK}\n{SHIM}\n");
    let probe = FakeShellProbe::new()
        .executable(NATIVE)
        .executable(CASK)
        .executable(SHIM)
        .answers(ShellMode::LoginInteractive, LOCATE, NATIVE)
        .answers(ShellMode::LoginInteractive, "which -a claude", &listing)
        .version(NATIVE, "2.1.330 (Claude Code)")
        .version(CASK, "2.1.280 (Claude Code)")
        .version(SHIM, "2.1.330 (Claude Code)");

    let inspection = inspect_claude(&InMemoryStore::new(), &probe);

    assert_eq!(inspection.chosen.unwrap(), PathBuf::from(NATIVE));
    let summary: Vec<(&str, Option<&str>, bool, bool)> = inspection
        .installations
        .iter()
        .map(|i| {
            (
                i.path.to_str().unwrap(),
                i.version.as_deref(),
                i.chosen,
                i.usable,
            )
        })
        .collect();
    assert_eq!(
        summary,
        vec![
            (NATIVE, Some("2.1.330"), true, true),
            (CASK, Some("2.1.280"), false, true),
            (SHIM, Some("2.1.330"), false, false),
        ]
    );
    assert!(inspection.version_mismatch);
}

// PLAN §0.6 doctor: one version, no warning
#[test]
fn doctor_does_not_warn_when_every_install_is_on_the_same_version() {
    let probe = FakeShellProbe::new()
        .executable(NATIVE)
        .executable(SHIM)
        .answers(ShellMode::LoginInteractive, LOCATE, NATIVE)
        .answers(
            ShellMode::LoginInteractive,
            "which -a claude",
            &format!("{NATIVE}\n{SHIM}\n"),
        )
        .version(NATIVE, "2.1.330 (Claude Code)")
        .version(SHIM, "2.1.280 (Claude Code)");
    assert!(!inspect_claude(&InMemoryStore::new(), &probe).version_mismatch);
}
