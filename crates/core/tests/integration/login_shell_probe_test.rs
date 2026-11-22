use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use lithemux_core::modules::launch::{LoginShellProbe, ShellMode, ShellProbe, resolve_claude};

use crate::support::in_memory_store::InMemoryStore;

fn fake(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fakes")
        .join(name)
}

/// The real probe over a fake shell. No folder counts as temporary, so what
/// the fake shell finds can be chosen wherever this repository is checked
/// out, a temporary folder included.
fn probe(shell: &str) -> LoginShellProbe {
    LoginShellProbe::with_temporary_roots(fake(shell), Vec::new())
}

// PLAN §0.3 order, step 2: found through the PATH of the interactive shell,
// with no fallback and no warning
#[test]
fn the_claude_on_the_path_of_the_interactive_shell_is_chosen_without_a_fallback() {
    for shell in ["fake-shell-lingering", "fake-shell-interactive"] {
        let resolved = resolve_claude(&InMemoryStore::new(), &probe(shell)).unwrap();

        assert_eq!(resolved.path.parent(), Some(&*fake("on-path")), "{shell}");
        assert_eq!(resolved.warning, None, "{shell}");
    }
}

// Audit of the delta (D-23): the answer is what the script printed, without
// what the shell printed before it, after it, or from a job
#[test]
fn the_probe_returns_only_what_the_script_printed() {
    for shell in ["fake-shell-trailing-output", "fake-shell-late-writer"] {
        let answer = probe(shell)
            .run(
                ShellMode::LoginInteractive,
                "echo answered",
                Duration::from_secs(3),
            )
            .unwrap();

        assert_eq!(answer.trim(), "answered", "{shell}");
    }
}

// PLAN §0.3: a shell that does not finish in time gives no answer, and the
// wait is the one that was asked for
#[test]
fn a_shell_that_hangs_gives_no_answer_within_the_wait() {
    let started = Instant::now();

    let answer = probe("fake-shell-hanging").run(
        ShellMode::LoginInteractive,
        "echo answered",
        Duration::from_millis(300),
    );

    assert_eq!(answer, None);
    assert!(started.elapsed() < Duration::from_secs(2));
}

// Audit of the delta (M2, D-29): a shell that fails in milliseconds has not
// used up the wait, so a slow fallback still gets the time to answer
#[test]
fn a_fast_failure_leaves_a_slow_fallback_the_time_to_answer() {
    let resolved = resolve_claude(&InMemoryStore::new(), &probe("fake-shell-slow-login")).unwrap();

    assert_eq!(resolved.path.parent(), Some(&*fake("on-path")));
    assert!(resolved.warning.unwrap().contains("fell back"));
}
