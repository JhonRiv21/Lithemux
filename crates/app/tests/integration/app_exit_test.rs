use std::process::{Child, Stdio};
use std::time::{Duration, Instant};

use lithemux_app::session_host::SessionHost;
use lithemux_core::modules::launch::{FlockSessionLocks, SessionLocks};
use lithemux_core::modules::session::Profile;
use rustix::process::{Pid, Signal, kill_process};

use crate::hosted_world::{HostedWorld, core_fake};
use crate::processes::{is_alive, wait_until_gone};
use crate::sandbox::app;

/// The real app, without a window, with one session of a fake claude that
/// ignores the hangup. Returns the app and the pid of that claude.
fn app_with_a_stubborn_session(world: &HostedWorld) -> (Child, String, i32) {
    let session = world.add_session("ejecutora", Profile::Claude);
    world.steer_fake_claude("fake-claude.sleep", "30");
    world.steer_fake_claude("fake-claude.ignore-hup", "");
    let child = app()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", world.dir.path())
        .env("SHELL", core_fake("fake-shell"))
        .env("LITHEMUX_HOME", world.home.path())
        .env("CLAUDE_CONFIG_DIR", world.claude_config())
        .args(["--no-window", "--open", "demo/ejecutora"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let (pid, _) = world.wait_for_run(&session.id, 1);
    (child, session.id, pid)
}

fn wait_for_exit(mut app: Child) -> String {
    let deadline = Instant::now() + Duration::from_secs(10);
    while app.try_wait().unwrap().is_none() {
        assert!(Instant::now() < deadline, "the app did not exit");
        std::thread::sleep(Duration::from_millis(10));
    }
    let output = app.wait_with_output().unwrap();
    String::from_utf8_lossy(&output.stderr).to_string()
}

// Audit of the §2.1 (I1): a signal ends the app the way cmd+q does, with its
// sessions closed first. A `claude` that ignores the hangup does not
// outlive it, the session is free and nothing of the app is left on disk.
// SIGHUP is what closing the terminal the app was started from sends.
#[test]
fn a_signal_closes_the_sessions_before_the_app_is_gone() {
    for signal in [Signal::TERM, Signal::INT, Signal::HUP] {
        let world = HostedWorld::new();
        let (app, session_id, claude) = app_with_a_stubborn_session(&world);
        assert!(is_alive(claude));
        assert!(world.home.app_socket_path().exists());

        kill_process(Pid::from_raw(app.id().try_into().unwrap()).unwrap(), signal).unwrap();
        let said = wait_for_exit(app);

        assert!(
            !is_alive(claude),
            "{signal:?}: claude outlived the app: {said}"
        );
        assert!(
            !world.home.app_socket_path().exists(),
            "{signal:?}: the socket was left behind"
        );
        let locks = FlockSessionLocks::new(&world.home);
        assert_eq!(locks.holder(&session_id), None, "{signal:?}");
        assert!(locks.acquire(&session_id).is_ok(), "{signal:?}");
        // The app's own lock is free too: another one can start.
        assert!(SessionHost::start(world.config()).is_ok(), "{signal:?}");
    }
}

// PLAN §2.1: a second instance leaves with a clear message, and the first
// one and its sessions are untouched
#[test]
fn a_second_app_on_the_same_home_leaves_and_disturbs_nothing() {
    let world = HostedWorld::new();
    let (first, _session, claude) = app_with_a_stubborn_session(&world);

    let second = app()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", world.dir.path())
        .env("LITHEMUX_HOME", world.home.path())
        .args(["--no-window", "--open", "demo/ejecutora"])
        .output()
        .unwrap();

    assert_eq!(second.status.code(), Some(1));
    let said = String::from_utf8_lossy(&second.stderr).to_string();
    assert!(
        said.contains(&format!("already running (pid {})", first.id())),
        "{said}"
    );
    assert!(is_alive(claude));
    assert!(world.home.app_socket_path().exists());

    kill_process(
        Pid::from_raw(first.id().try_into().unwrap()).unwrap(),
        Signal::TERM,
    )
    .unwrap();
    wait_for_exit(first);
    wait_until_gone(claude);
}

// Audit of the §2.1 (7): what is wrong on the command line is said, not
// dropped
#[test]
fn a_command_line_that_makes_no_sense_is_refused() {
    for arguments in [
        &["--open"][..],
        &["--open", "demo/uno", "stray"],
        &["--open", "demo/uno", "--open", "demo/uno"],
    ] {
        let output = app().args(arguments).output().unwrap();

        assert_eq!(output.status.code(), Some(2), "{arguments:?}");
        assert!(
            String::from_utf8_lossy(&output.stderr).starts_with("lithemux-app: "),
            "{arguments:?}"
        );
    }
}
