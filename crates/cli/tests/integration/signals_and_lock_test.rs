use std::fs;
use std::time::Duration;

use rustix::process::Signal;

use crate::sandbox::Sandbox;
use crate::signals;

fn sleeping_claude(seconds: &str) -> Sandbox {
    let sandbox = Sandbox::new().with_demo();
    sandbox.use_fake_claude();
    fs::write(sandbox.home().join("fake-claude.sleep"), seconds).unwrap();
    sandbox
}

// docs/decisiones.md D-03: the lock is not inherited, so something `claude`
// leaves running cannot keep the session closed
#[test]
fn a_process_that_outlives_claude_does_not_keep_the_session_locked() {
    let sandbox = Sandbox::new().with_demo();
    sandbox.use_fake_claude();
    fs::write(sandbox.home().join("fake-claude.leave-behind"), "10").unwrap();

    sandbox.run(&["open", "demo/ejecutora"]);

    let left_behind = sandbox.fake_claude_number("fake-claude.left-behind.pid");
    assert!(signals::is_alive(left_behind));
    assert!(!sandbox.run(&["status"]).contains("open (pid"));
    sandbox.run(&["open", "demo/ejecutora"]);
    signals::send(left_behind.try_into().unwrap(), Signal::KILL);
}

// docs/decisiones.md D-03: SIGTERM and SIGHUP reach the child, and `open`
// reports how it ended
#[test]
fn sigterm_and_sighup_are_forwarded_to_claude() {
    for (signal, code) in [(Signal::TERM, 143), (Signal::HUP, 129)] {
        let sandbox = sleeping_claude("10");
        let mut open = sandbox.open_in_background("demo/ejecutora");
        let claude = sandbox.fake_claude_number("fake-claude.pid");

        signals::send(open.id(), signal);

        assert_eq!(open.wait().unwrap().code(), Some(code), "{signal:?}");
        signals::wait_until_gone(claude);
    }
}

// docs/decisiones.md D-03: ctrl+c belongs to claude, not to the launcher
#[test]
fn sigint_does_not_end_the_launcher() {
    let sandbox = sleeping_claude("1");
    fs::write(sandbox.home().join("fake-claude.exit"), "7").unwrap();
    let mut open = sandbox.open_in_background("demo/ejecutora");

    signals::send(open.id(), Signal::INT);
    std::thread::sleep(Duration::from_millis(200));

    assert!(open.try_wait().unwrap().is_none(), "the launcher is alive");
    assert_eq!(open.wait().unwrap().code(), Some(7));
}
