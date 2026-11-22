use std::time::{Duration, Instant};

use lithemux_app::session_host::{Opened, Phase, SessionHost};
use lithemux_app::terminal_view::GridSize;
use lithemux_core::modules::session::SessionRef;

use crate::sandbox::Sandbox;
use crate::signals;

/// Asks the app for the session and waits until it knows how it went.
fn open_in_the_app(host: &mut SessionHost, target: &str) -> (String, Phase) {
    let asked = host
        .open(&SessionRef::parse(target).unwrap(), GridSize::INITIAL)
        .unwrap();
    let (Opened::Opening { session_id } | Opened::AlreadyHere { session_id }) = asked;
    attend_until(host, "the session to stop opening", |host| {
        host.phase(&session_id) != Some(&Phase::Opening)
    });
    let phase = host.phase(&session_id).cloned().unwrap();
    (session_id, phase)
}

fn status_line(sandbox: &Sandbox, name: &str) -> String {
    sandbox
        .run(&["status"])
        .lines()
        .find(|line| line.contains(name))
        .unwrap()
        .to_string()
}

fn attend_until(host: &mut SessionHost, what: &str, mut done: impl FnMut(&SessionHost) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        host.process_pending();
        if done(host) {
            return;
        }
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(5));
    }
}

// PLAN §2.1 closing criteria: a session the app has open cannot be opened
// with `lithemux open`, and `lithemux status` shows who has it
#[test]
fn the_cli_cannot_open_what_the_app_has_open_and_sees_it_in_status() {
    let sandbox = Sandbox::new().with_demo();
    sandbox.use_fake_claude();
    sandbox.steer_fake_claude("fake-claude.sleep", "30");
    let mut host = sandbox.app_host();

    let (_, phase) = open_in_the_app(&mut host, "demo/ejecutora");

    assert_eq!(phase, Phase::Running);
    sandbox.wait_for_fake_claude();
    let claude = sandbox.fake_claude_number("fake-claude.pid");
    let refused = sandbox.fail(&["open", "demo/ejecutora"]);
    assert!(refused.contains("already open"), "{refused}");
    assert!(refused.contains(&format!("(pid {claude})")), "{refused}");
    let line = status_line(&sandbox, "demo-ejecutora");
    assert!(line.contains(&format!("open (pid {claude})")), "{line}");

    // The other sessions of the category are free.
    assert!(status_line(&sandbox, "demo-documental").contains("closed"));
    drop(host);
    signals::wait_until_gone(claude);
    assert!(status_line(&sandbox, "demo-ejecutora").contains("closed"));
}

// PLAN §2.1 closing criteria: `lithemux status` sees the live state of the
// sessions of the app
#[test]
fn status_follows_a_session_of_the_app_to_its_end() {
    let sandbox = Sandbox::new().with_demo();
    sandbox.use_fake_claude();
    let mut host = sandbox.app_host();
    let (session_id, phase) = open_in_the_app(&mut host, "demo/ejecutora");
    assert_eq!(phase, Phase::Running);

    // The fake claude exits at once; the host notices and says so.
    attend_until(&mut host, "the end of the session", |host| {
        !host.is_running(&session_id)
    });

    let line = status_line(&sandbox, "demo-ejecutora");
    assert!(line.contains("ended"), "{line}");
    assert!(line.contains("closed"), "{line}");
}

// PLAN §2.1 closing criteria: and the other way around, the app does not
// launch what `lithemux open` has open; it says who has it
#[test]
fn the_app_does_not_launch_what_the_cli_has_open() {
    let sandbox = Sandbox::new().with_demo();
    sandbox.use_fake_claude();
    sandbox.steer_fake_claude("fake-claude.sleep", "30");
    let mut cli = sandbox.open_in_background("demo/ejecutora");
    let claude = sandbox.fake_claude_number("fake-claude.pid");
    let mut host = sandbox.app_host();

    let (_, phase) = open_in_the_app(&mut host, "demo/ejecutora");

    assert_eq!(
        phase,
        Phase::OpenElsewhere {
            pid: Some(claude.try_into().unwrap())
        }
    );
    // Nothing was started over the conversation of the CLI.
    assert_eq!(sandbox.fake_claude_number("fake-claude.pid"), claude);
    assert!(signals::is_alive(claude));

    signals::send(cli.id(), rustix::process::Signal::TERM);
    cli.wait().unwrap();
    signals::wait_until_gone(claude);
    // Once the CLI lets go, the app can have it.
    assert_eq!(
        open_in_the_app(&mut host, "demo/ejecutora").1,
        Phase::Running
    );
}
