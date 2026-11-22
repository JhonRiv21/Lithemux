use std::fs;

use lithemux_app::session_host::{HostError, Opened, Phase, SessionHost};
use lithemux_core::modules::launch::{FlockSessionLocks, LaunchError, SessionLocks};
use lithemux_core::modules::session::Profile;

use crate::hosted_world::{HostedWorld, open, run, target};
use crate::support::{SIZE, wait_for};

// PLAN §2.1: the app launches with the rules of `core::launch`, in a PTY
#[test]
fn a_session_runs_in_a_terminal_of_the_host() {
    let world = HostedWorld::new();
    let session = world.add_session("ejecutora", Profile::Claude);
    let (mut host, _changes) = world.start();

    let id = run(&mut host, &world, "ejecutora");

    assert_eq!(id, session.id);
    assert!(host.is_running(&id));
    let (terminal, _wakes) = host.terminal(&id).unwrap();
    wait_for(&terminal, "fake claude ready");
    let (_, flag) = world.wait_for_run(&id, 1);
    assert_eq!(flag, "--session-id");
    let log = fs::read_to_string(world.home.path().join("fake-claude.log")).unwrap();
    assert!(
        log.contains(&format!("cwd={}", world.work().display())),
        "{log}"
    );
    assert!(log.contains("arg=demo-ejecutora"), "{log}");
    let terminal_of_claude =
        fs::read_to_string(world.home.path().join("fake-claude.terminal")).unwrap();
    assert!(terminal_of_claude.contains("foreground=yes"));
}

// PLAN §2.1: a hidden terminal goes on parsing what its program prints
#[test]
fn a_session_nobody_looks_at_keeps_its_screen_up_to_date() {
    let world = HostedWorld::new();
    world.add_session("ejecutora", Profile::Claude);
    world.steer_fake_claude("fake-claude.late-line", "");
    world.steer_fake_claude("fake-claude.sleep", "30");
    let (mut host, _changes) = world.start();
    let id = run(&mut host, &world, "ejecutora");

    // No view was ever attached: nothing reads the wakes.
    let (terminal, _wakes) = host.terminal(&id).unwrap();

    wait_for(&terminal, "a line printed later");
}

// PLAN §2.1 closing criteria: a session the app has open cannot be opened

// by anyone else, and they are told who has it
#[test]
fn the_host_holds_the_lock_of_a_running_session() {
    let world = HostedWorld::new();
    world.add_session("ejecutora", Profile::Claude);
    world.steer_fake_claude("fake-claude.sleep", "30");
    let (mut host, _changes) = world.start();
    let id = run(&mut host, &world, "ejecutora");
    let (pid, _) = world.wait_for_run(&id, 1);

    let locks = FlockSessionLocks::new(&world.home);

    assert!(matches!(
        locks.acquire(&id),
        Err(LaunchError::AlreadyOpen { pid: Some(holder) }) if i64::from(holder) == i64::from(pid)
    ));
    assert_eq!(locks.holder(&id).map(i64::from), Some(i64::from(pid)));
}

// PLAN §2.1: a session open in another terminal is shown as such, with the

// pid of its owner, and is not launched
#[test]
fn a_session_open_in_another_terminal_is_not_launched() {
    let world = HostedWorld::new();
    let session = world.add_session("ejecutora", Profile::Claude);
    let (mut host, _changes) = world.start();
    let mut elsewhere = FlockSessionLocks::new(&world.home)
        .acquire(&session.id)
        .unwrap();
    elsewhere.record_pid(std::process::id());

    let opened = open(&mut host, "ejecutora");

    assert_eq!(
        opened,
        Phase::OpenElsewhere {
            pid: Some(std::process::id())
        }
    );
    assert!(!host.is_running(&session.id));
    assert!(host.terminal(&session.id).is_none());
    assert!(world.fake_claude_runs().is_empty());
}

// PLAN §2.1: opening what already runs here starts nothing new
#[test]
fn opening_a_session_twice_starts_it_once() {
    let world = HostedWorld::new();
    world.add_session("ejecutora", Profile::Claude);
    world.add_session("term", Profile::Shell);
    world.steer_fake_claude("fake-claude.sleep", "30");
    let (mut host, _changes) = world.start();
    let id = run(&mut host, &world, "ejecutora");
    let shell = run(&mut host, &world, "term");
    world.wait_for_run(&id, 1);

    for (slug, session_id) in [("ejecutora", id), ("term", shell)] {
        assert_eq!(
            host.open(&target(slug), SIZE).unwrap(),
            Opened::AlreadyHere { session_id }
        );
    }
    assert_eq!(world.fake_claude_runs().len(), 1);
}

// PLAN §2.1 closing criteria: a hook event reaches the app and changes the

// PLAN §2.1: one instance of the app per home
#[test]
fn a_second_host_on_the_same_home_is_refused() {
    let world = HostedWorld::new();
    let (first, _changes) = world.start();

    let second = SessionHost::start(world.config());

    assert!(matches!(
        second.err(),
        Some(HostError::AlreadyRunning { pid: Some(pid) }) if pid == std::process::id()
    ));
    drop(first);
    assert!(SessionHost::start(world.config()).is_ok());
}

// PLAN §2.1: a socket left behind by an app that died is replaced, and the

// PLAN §2.1: a shell session runs in the directory of the session, with no

// hooks and no lock
#[test]
fn a_shell_session_runs_in_its_directory_without_a_lock() {
    let world = HostedWorld::new();
    let session = world.add_session("term", Profile::Shell);
    let (mut host, _changes) = world.start();

    let id = run(&mut host, &world, "term");

    let (terminal, _wakes) = host.terminal(&id).unwrap();
    terminal.write(b"pwd -P; echo session=$LITHEMUX_SESSION_ID\n".to_vec());
    let screen = wait_for(&terminal, &format!("session={}", session.id));
    assert!(
        screen.contains(&world.work().display().to_string()),
        "{screen}"
    );
    assert_eq!(FlockSessionLocks::new(&world.home).holder(&id), None);
    assert!(world.fake_claude_runs().is_empty());
}

// PLAN §2.1 closing criteria: six sessions alive at once
#[test]
fn six_sessions_live_side_by_side() {
    let world = HostedWorld::new();
    world.steer_fake_claude("fake-claude.sleep", "30");
    let (mut host, _changes) = world.start();
    let names = ["a", "b", "c", "d", "e", "f"];
    let ids: Vec<String> = names
        .iter()
        .map(|name| {
            world.add_session(name, Profile::Claude);
            run(&mut host, &world, name)
        })
        .collect();

    for id in &ids {
        let (terminal, _wakes) = host.terminal(id).unwrap();
        wait_for(&terminal, "fake claude ready");
        assert!(host.is_running(id));
    }
    let pids: Vec<i32> = ids.iter().map(|id| world.wait_for_run(id, 1).0).collect();
    assert_eq!(pids.len(), 6);
}

// Audit of the §2.1 (5): clients that connect and say nothing, or say it

// Audit M9: a socket somebody answers on is not a leftover; it is not
