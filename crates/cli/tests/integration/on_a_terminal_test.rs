use std::time::{Duration, Instant};

use crate::sandbox::Sandbox;
use crate::signals;

/// Well under the three seconds the lookup waits for the shell.
const LIMIT: Duration = Duration::from_secs(2);

fn on_a_terminal() -> Sandbox {
    Sandbox::new()
        .with_fake_shell("fake-shell-interactive")
        .with_demo()
}

// PLAN §0.3 order, step 2 + §2.0 bug 1: in real use there is always a
// terminal, and the interactive shell that is asked for `claude` touches it
#[test]
fn doctor_on_a_terminal_finds_claude_through_the_interactive_shell() {
    let sandbox = on_a_terminal();
    let started = Instant::now();

    let report = sandbox.run_on_a_terminal(&["doctor"]);

    assert!(started.elapsed() < LIMIT, "took {:?}", started.elapsed());
    // Listed by the interactive shell, wherever the repository lives.
    assert!(report.contains("fakes/on-path/"), "{report}");
    if Sandbox::fakes_on_the_path_are_launchable() {
        assert!(!report.contains("fell back"), "{report}");
    }
}

// PLAN §0.3 order, step 2: the same lookup, for a launch
#[test]
fn open_on_a_terminal_launches_the_claude_of_the_interactive_shell() {
    let sandbox = on_a_terminal();
    let started = Instant::now();

    let printed = sandbox.run_on_a_terminal(&["open", "demo/ejecutora"]);

    assert!(started.elapsed() < LIMIT, "took {:?}", started.elapsed());
    if Sandbox::fakes_on_the_path_are_launchable() {
        assert!(sandbox.home().join("fake-claude.log").exists(), "{printed}");
        assert!(!printed.contains("fell back"), "{printed}");
    }
}

// docs/decisiones.md D-03: the session takes over the terminal `open` was
// run from, in the foreground, where it can read keys and change settings
#[test]
fn claude_gets_the_terminal_in_the_foreground() {
    let sandbox = Sandbox::new().with_demo();
    sandbox.use_fake_claude();

    let (code, printed) = sandbox
        .start_on_a_terminal(&["open", "demo/ejecutora"])
        .finish();

    assert_eq!(code, 0, "{printed}");
    let terminal = std::fs::read_to_string(sandbox.home().join("fake-claude.terminal")).unwrap();
    assert_eq!(
        terminal.lines().collect::<Vec<_>>(),
        ["stdin=terminal", "stdout=terminal", "foreground=yes"]
    );
}

// docs/decisiones.md D-03: ctrl+c typed on the terminal reaches claude, and
// `open` survives it to report how the session ended
#[test]
fn ctrl_c_on_the_terminal_reaches_claude_and_open_reports_it() {
    let sandbox = Sandbox::new().with_demo();
    sandbox.use_fake_claude();
    std::fs::write(sandbox.home().join("fake-claude.sleep"), "10").unwrap();
    let mut open = sandbox.start_on_a_terminal(&["open", "demo/ejecutora"]);
    sandbox.wait_for_fake_claude();
    let claude = sandbox.fake_claude_number("fake-claude.pid");
    let started = Instant::now();

    open.type_keys(b"\x03");
    let (code, printed) = open.finish();

    assert_eq!(code, 130, "{printed}");
    assert!(started.elapsed() < LIMIT, "took {:?}", started.elapsed());
    signals::wait_until_gone(claude);
}
