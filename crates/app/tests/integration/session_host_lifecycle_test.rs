//! PLAN §2.1: a session's program in the host, from its start to its end.

use std::fs;

use lithemux_core::modules::launch::{FlockSessionLocks, SessionLocks};
use lithemux_core::modules::session::{Profile, State};

use crate::hosted_world::{HostedWorld, attend_until, run};
use crate::processes::{is_alive, wait_until_gone};
use crate::support::wait_for;

// PLAN §2.1: when `claude` exits by itself the session ends, its screen
// stays, and reopening it resumes the same conversation
#[test]
fn a_claude_that_exits_leaves_an_ended_session_that_can_be_resumed() {
    let world = HostedWorld::new();
    let session = world.add_session("ejecutora", Profile::Claude);
    let (mut host, _changes) = world.start();
    let id = run(&mut host, &world, "ejecutora");
    let (first_pid, _) = world.wait_for_run(&id, 1);

    attend_until(&mut host, "the end of the session", |host| {
        !host.is_running(&id)
    });

    assert_eq!(host.state(&id), Some(State::Ended));
    assert_eq!(world.stored(&id).state, State::Ended);
    assert_eq!(FlockSessionLocks::new(&world.home).holder(&id), None);
    let (terminal, _wakes) = host.terminal(&id).unwrap();
    wait_for(&terminal, "fake claude ready");
    wait_until_gone(first_pid);

    // What Claude Code does on a real first run: it writes the transcript.
    let conversation = session.claude_session_id.unwrap();
    let project = world.claude_config().join("projects/-work");
    fs::create_dir_all(&project).unwrap();
    fs::write(project.join(format!("{conversation}.jsonl")), "{}\n").unwrap();

    assert_eq!(run(&mut host, &world, "ejecutora"), id);
    let (_, flag) = world.wait_for_run(&id, 2);
    assert_eq!(flag, "--resume");
}

// PLAN §2.1 closing criteria: closing the app kills its `claude`s, even one
// that ignores the hangup, and frees their sessions
#[test]
fn shutting_down_leaves_no_program_behind() {
    let world = HostedWorld::new();
    world.add_session("uno", Profile::Claude);
    world.add_session("dos", Profile::Claude);
    world.steer_fake_claude("fake-claude.sleep", "30");
    world.steer_fake_claude("fake-claude.ignore-hup", "");
    let (mut host, _changes) = world.start();
    let ids = [run(&mut host, &world, "uno"), run(&mut host, &world, "dos")];
    let pids: Vec<i32> = ids.iter().map(|id| world.wait_for_run(id, 1).0).collect();
    assert!(pids.iter().all(|pid| is_alive(*pid)));

    drop(host);

    for pid in pids {
        assert!(!is_alive(pid), "process {pid} outlived the host");
    }
    let locks = FlockSessionLocks::new(&world.home);
    for id in &ids {
        assert_eq!(locks.holder(id), None);
    }
}

// PLAN §2.1: closing one session ends its program and nobody else's
#[test]
fn closing_a_session_ends_only_its_program() {
    let world = HostedWorld::new();
    world.add_session("uno", Profile::Claude);
    world.add_session("dos", Profile::Claude);
    world.steer_fake_claude("fake-claude.sleep", "30");
    let (mut host, _changes) = world.start();
    let uno = run(&mut host, &world, "uno");
    let dos = run(&mut host, &world, "dos");
    let (pid_uno, _) = world.wait_for_run(&uno, 1);
    let (pid_dos, _) = world.wait_for_run(&dos, 1);

    host.close(&uno);

    wait_until_gone(pid_uno);
    assert!(is_alive(pid_dos));
    assert!(!host.is_running(&uno));
    assert!(host.is_running(&dos));
    assert!(host.terminal(&uno).is_none());
}

// PLAN §2.1 (audit M2): the end of the program does not undo a hibernation
// that was decided on purpose
#[test]
fn a_program_that_ends_leaves_a_hibernated_session_hibernated() {
    let world = HostedWorld::new();
    world.add_session("ejecutora", Profile::Claude);
    world.steer_fake_claude("fake-claude.sleep", "30");
    let (mut host, _changes) = world.start();
    let id = run(&mut host, &world, "ejecutora");
    let (pid, _) = world.wait_for_run(&id, 1);
    // What the app will do before it stops a session to save memory.
    world.set_state(&id, State::Hibernated);

    kill(pid);
    attend_until(&mut host, "the end of the session", |host| {
        !host.is_running(&id)
    });

    assert_eq!(world.stored(&id).state, State::Hibernated);
    assert_eq!(host.state(&id), Some(State::Hibernated));
}

// Audit M16: a session that ended here and is opened again has one terminal,
// the one of the new run
#[test]
fn opening_again_replaces_the_terminal_of_the_run_that_ended() {
    let world = HostedWorld::new();
    world.add_session("ejecutora", Profile::Claude);
    let (mut host, _changes) = world.start();
    let id = run(&mut host, &world, "ejecutora");
    attend_until(&mut host, "the end of the first run", |host| {
        !host.is_running(&id)
    });
    let (ended, _wakes) = host.terminal(&id).unwrap();
    assert!(ended.has_exited());

    world.steer_fake_claude("fake-claude.sleep", "30");
    run(&mut host, &world, "ejecutora");

    let (current, _wakes) = host.terminal(&id).unwrap();
    assert!(!std::rc::Rc::ptr_eq(&ended, &current));
    assert!(!current.process_is_gone());
}

// Audit of the §2.1 (6): the session is free the moment its program is
// gone, whether or not anyone attends to the host
#[test]
fn the_lock_is_released_when_the_program_is_gone_without_the_host_noticing() {
    let world = HostedWorld::new();
    world.add_session("ejecutora", Profile::Claude);
    let (mut host, _changes) = world.start();
    let id = run(&mut host, &world, "ejecutora");
    let (pid, _) = world.wait_for_run(&id, 1);

    // Nobody calls `process_pending` from here on.
    wait_until_gone(pid);
    let locks = FlockSessionLocks::new(&world.home);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while locks.holder(&id).is_some() {
        assert!(std::time::Instant::now() < deadline, "the lock was kept");
    }

    assert!(locks.acquire(&id).is_ok());
}

// Audit of the §2.1 (6): and it is not free one moment sooner, even after
// the session was closed in the app
#[test]
fn a_closed_session_stays_locked_until_its_stubborn_program_is_gone() {
    let world = HostedWorld::new();
    world.add_session("ejecutora", Profile::Claude);
    world.steer_fake_claude("fake-claude.sleep", "30");
    world.steer_fake_claude("fake-claude.ignore-hup", "");
    let (mut host, _changes) = world.start();
    let id = run(&mut host, &world, "ejecutora");
    let (pid, _) = world.wait_for_run(&id, 1);
    let locks = FlockSessionLocks::new(&world.home);

    host.close(&id);

    // It ignores the hangup, so it lives through the grace period, locked.
    assert!(is_alive(pid));
    assert!(locks.acquire(&id).is_err());
    wait_until_gone(pid);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while locks.acquire(&id).is_err() {
        assert!(std::time::Instant::now() < deadline, "the lock was kept");
    }
}

fn kill(pid: i32) {
    use rustix::process::{Pid, Signal, kill_process};
    kill_process(Pid::from_raw(pid).unwrap(), Signal::KILL).unwrap();
}

// Audit of block 3 (3): a session removed from the database while the app
// runs it (`lithemux rm`, or its category) does not go on out of sight.
#[test]
fn a_session_removed_from_the_database_is_closed_here() {
    let world = HostedWorld::new();
    world.add_session("borrada", Profile::Claude);
    world.add_session("shell", Profile::Shell);
    world.add_session("queda", Profile::Claude);
    world.steer_fake_claude("fake-claude.sleep", "30");
    let (mut host, _changes) = world.start();
    let removed = [
        run(&mut host, &world, "borrada"),
        run(&mut host, &world, "shell"),
    ];
    let kept = run(&mut host, &world, "queda");
    let (pid, _) = world.wait_for_run(&removed[0], 1);
    assert!(!host.close_removed());

    for id in &removed {
        world.remove(id);
    }
    assert!(host.close_removed());

    attend_until(&mut host, "the removed sessions to be closed", |host| {
        removed.iter().all(|id| host.phase(id).is_none())
    });
    wait_until_gone(pid);
    assert!(host.is_running(&kept));
    assert_eq!(
        FlockSessionLocks::new(&world.home).holder(&removed[0]),
        None
    );
}
