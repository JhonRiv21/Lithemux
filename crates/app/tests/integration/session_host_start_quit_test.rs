//! PLAN §2.5: what opening and closing the app do to the sessions.

use std::fs;

use lithemux_core::modules::launch::{FlockSessionLocks, SessionLocks};
use lithemux_core::modules::session::{Profile, State};

use crate::hosted_world::{HostedWorld, run};
use crate::processes::is_alive;

// PLAN §2.5 and principle 3: closing the app is hibernating what runs.
#[test]
fn shutting_down_leaves_the_running_sessions_hibernated() {
    let world = HostedWorld::new();
    world.add_session("uno", Profile::Claude);
    world.add_session("acabada", Profile::Claude);
    world.steer_fake_claude("fake-claude.sleep", "30");
    let (mut host, _changes) = world.start();
    let running = run(&mut host, &world, "uno");
    let closed = run(&mut host, &world, "acabada");
    world.wait_for_run(&closed, 1);
    host.close(&closed);

    drop(host);

    assert_eq!(world.stored(&running).state, State::Hibernated);
    // The one the user closed is ended, not asleep.
    assert_eq!(world.stored(&closed).state, State::Ended);
}

// PLAN §2.5: when the app opens, every Claude session that was running
// appears hibernated, whatever the database was left saying.
#[test]
fn starting_the_app_hibernates_what_a_crash_left_as_running() {
    let world = HostedWorld::new();
    let stale = world.add_session("colgada", Profile::Claude);
    let ended = world.add_session("terminada", Profile::Claude);
    let elsewhere = world.add_session("en-otra-terminal", Profile::Claude);
    world.set_state(&stale.id, State::Working);
    world.set_state(&ended.id, State::Ended);
    world.set_state(&elsewhere.id, State::Idle);
    let locks = FlockSessionLocks::new(&world.home);
    let _held = locks.acquire(&elsewhere.id).unwrap();

    let (host, _changes) = world.start();

    assert_eq!(world.stored(&stale.id).state, State::Hibernated);
    assert_eq!(world.stored(&ended.id).state, State::Ended);
    assert_eq!(world.stored(&elsewhere.id).state, State::Idle);
    drop(host);
}

// Audit of block 4 (1): quitting the app while a session is still within
// the grace of its hibernation must not leave its program alive. An orphan
// `claude` without its lock would write to the same conversation as the
// next one to open the session.
#[test]
fn shutting_down_during_the_grace_of_a_hibernation_leaves_no_program_alive() {
    let world = HostedWorld::new();
    world.add_session("dormilona", Profile::Claude);
    world.steer_fake_claude("fake-claude.sleep", "30");
    world.steer_fake_claude("fake-claude.ignore-hup", "");
    let mut config = world.config();
    // Longer than leaving the app used to wait.
    config.hibernate_grace = std::time::Duration::from_secs(4);
    let (mut host, _changes) = lithemux_app::session_host::SessionHost::start(config).unwrap();
    let id = run(&mut host, &world, "dormilona");
    let (pid, _) = world.wait_for_run(&id, 1);
    assert!(host.hibernate(&id));

    drop(host);

    assert!(!is_alive(pid), "program {pid} outlived the app");
    assert_eq!(FlockSessionLocks::new(&world.home).holder(&id), None);
    assert_eq!(world.stored(&id).state, State::Hibernated);
}

// Audit of block 4 (6): what decides whether a session runs elsewhere is
// its lock, not the pid written in the lock's file. A lock that is held
// with nothing written yet (the moment between taking it and recording the
// pid) is somebody's: the session is left alone.
#[test]
fn starting_the_app_leaves_alone_a_session_whose_lock_is_held() {
    let world = HostedWorld::new();
    let elsewhere = world.add_session("en-otra-terminal", Profile::Claude);
    world.set_state(&elsewhere.id, State::Working);
    let locks = FlockSessionLocks::new(&world.home);
    let _held = locks.acquire(&elsewhere.id).unwrap();
    fs::write(world.home.lock_path(&elsewhere.id), "").unwrap();
    assert!(locks.acquire(&elsewhere.id).is_err(), "the lock is held");

    let (host, _changes) = world.start();

    assert_eq!(world.stored(&elsewhere.id).state, State::Working);
    drop(host);
}
