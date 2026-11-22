use std::fs;

use lithemux_core::modules::launch::{FlockSessionLocks, SessionLocks};
use lithemux_core::modules::session::{Pressure, Profile, State};

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

fn write_transcript(world: &HostedWorld, conversation: &str) {
    let project = world.claude_config().join("projects/-work");
    fs::create_dir_all(&project).unwrap();
    fs::write(project.join(format!("{conversation}.jsonl")), "{}\n").unwrap();
}

// PLAN §2.4: hibernating writes `state=hibernated`, hangs up on the process
// group and kills it after the grace if it is still alive. Waking goes
// through the launcher, which resumes the same conversation.
#[test]
fn a_hibernated_session_stops_frees_its_lock_and_wakes_with_its_conversation() {
    let world = HostedWorld::new();
    let session = world.add_session("dormilona", Profile::Claude);
    world.steer_fake_claude("fake-claude.sleep", "30");
    world.steer_fake_claude("fake-claude.ignore-hup", "");
    let (mut host, _changes) = world.start();
    let id = run(&mut host, &world, "dormilona");
    let (pid, _) = world.wait_for_run(&id, 1);
    write_transcript(&world, &session.claude_session_id.unwrap());

    assert!(host.hibernate(&id));
    // Written at once, before the program is gone.
    assert_eq!(world.stored(&id).state, State::Hibernated);
    assert!(!host.is_running(&id));

    // It ignores the hangup: the kill after the grace ends it.
    wait_until_gone(pid);
    attend_until(&mut host, "the session to be free", |host| {
        host.phase(&id).is_none()
    });
    assert_eq!(world.stored(&id).state, State::Hibernated);
    assert_eq!(FlockSessionLocks::new(&world.home).holder(&id), None);

    assert_eq!(run(&mut host, &world, "dormilona"), id);
    let (_, flag) = world.wait_for_run(&id, 2);
    assert_eq!(flag, "--resume");
}

// PLAN §2.4
#[test]
fn only_a_running_claude_session_can_be_hibernated() {
    let world = HostedWorld::new();
    world.add_session("shell", Profile::Shell);
    let closed = world.add_session("cerrada", Profile::Claude);
    let (mut host, _changes) = world.start();
    let shell = run(&mut host, &world, "shell");
    assert!(!host.hibernate(&shell));
    assert!(host.is_running(&shell));
    assert!(!host.hibernate(&closed.id));
    assert!(!host.hibernate("no-such-session"));
    assert_eq!(world.stored(&closed.id).state, State::New);
}

/// Three sessions that run: idle for `hours` each, by the database.
fn idle_sessions(
    world: &HostedWorld,
    host: &mut lithemux_app::session_host::SessionHost,
    hours: &[(&str, i64)],
    now: i64,
) -> Vec<String> {
    hours
        .iter()
        .map(|(slug, idle_for)| {
            world.add_session(slug, Profile::Claude);
            let id = run(host, world, slug);
            world.set_idle_since(&id, now - idle_for * 3600);
            id
        })
        .collect()
}

// PLAN §2.4: idle, out of sight, not kept alive and quiet for more than the
// hours. The clock is given: nobody waits two days.
#[test]
fn the_sessions_that_are_due_are_hibernated_and_the_others_keep_running() {
    let now = 1_900_000_000;
    let world = HostedWorld::new();
    world.steer_fake_claude("fake-claude.sleep", "30");
    let (mut host, _changes) = world.start();
    let ids = idle_sessions(
        &world,
        &mut host,
        &[
            ("vieja", 50),
            ("visible", 50),
            ("viva", 50),
            ("reciente", 2),
        ],
        now,
    );
    world.keep_alive(&ids[2]);
    world.add_session("ocupada", Profile::Claude);
    let busy = run(&mut host, &world, "ocupada");
    world.set_idle_since(&busy, now - 50 * 3600);
    world.set_state(&busy, State::Working);

    let hibernated = host.hibernate_what_is_due(std::slice::from_ref(&ids[1]), now, 48);

    assert_eq!(hibernated, [ids[0].clone()]);
    assert_eq!(world.stored(&ids[0]).state, State::Hibernated);
    for still in [&ids[1], &ids[2], &ids[3], &busy] {
        assert!(host.is_running(still));
    }
    // With fewer hours configured, the recent one is due too.
    let hibernated = host.hibernate_what_is_due(std::slice::from_ref(&ids[1]), now, 1);
    assert_eq!(hibernated, [ids[3].clone()]);
}

// PLAN §2.4: the memory valve takes the idle hidden session that has been
// quiet longest, at most one every 60 s.
#[test]
fn the_memory_valve_takes_one_session_a_minute_starting_with_the_quietest() {
    let now = 1_900_000_000;
    let world = HostedWorld::new();
    world.steer_fake_claude("fake-claude.sleep", "30");
    let (mut host, _changes) = world.start();
    let ids = idle_sessions(&world, &mut host, &[("a", 1), ("b", 5), ("c", 3)], now);
    let nothing_visible: [String; 0] = [];

    assert_eq!(
        host.relieve_memory(&nothing_visible, now, Pressure::Critical),
        Some(ids[1].clone())
    );
    assert_eq!(
        host.relieve_memory(&nothing_visible, now + 30, Pressure::Critical),
        None
    );
    assert!(host.is_running(&ids[2]));
    assert_eq!(
        host.relieve_memory(&nothing_visible, now + 60, Pressure::Critical),
        Some(ids[2].clone())
    );
    // The last one is on show: the valve leaves it.
    assert_eq!(
        host.relieve_memory(std::slice::from_ref(&ids[0]), now + 200, Pressure::Critical),
        None
    );
    assert!(host.is_running(&ids[0]));
}

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

// Audit of block 4 (9): the kill of a hibernation comes after the grace
// that was configured, not before: `claude` gets that long to leave.
#[test]
fn a_hibernated_program_gets_the_whole_grace_before_it_is_killed() {
    let world = HostedWorld::new();
    world.add_session("terca", Profile::Claude);
    world.steer_fake_claude("fake-claude.sleep", "30");
    world.steer_fake_claude("fake-claude.ignore-hup", "");
    let mut config = world.config();
    config.hibernate_grace = std::time::Duration::from_millis(1500);
    let (mut host, _changes) = lithemux_app::session_host::SessionHost::start(config).unwrap();
    let id = run(&mut host, &world, "terca");
    let (pid, _) = world.wait_for_run(&id, 1);

    let asked = std::time::Instant::now();
    assert!(host.hibernate(&id));
    // Well past the grace of a plain close (200 ms), well within this one.
    std::thread::sleep(std::time::Duration::from_millis(900));
    assert!(
        is_alive(pid),
        "killed {:?} into a 1.5 s grace",
        asked.elapsed()
    );

    wait_until_gone(pid);
    assert!(asked.elapsed() >= std::time::Duration::from_millis(1500));
    std::mem::forget(host);
}

// PLAN §2.4, adjustment of 22-nov: at «warning» the valve takes only what
// has been quiet for half an hour; a session that waits for the user is
// never taken by the valve, but hibernates after the hours like an idle one.
#[test]
fn at_warning_the_valve_spares_the_recent_and_the_hours_take_the_waiting() {
    let now = 1_900_000_000;
    let world = HostedWorld::new();
    world.steer_fake_claude("fake-claude.sleep", "30");
    let (mut host, _changes) = world.start();
    let ids = idle_sessions(
        &world,
        &mut host,
        &[("reciente", 0), ("esperando", 60)],
        now,
    );
    world.set_idle_since(&ids[0], now - 10 * 60);
    world.set_state(&ids[1], State::Waiting);
    let nothing_visible: [String; 0] = [];

    assert_eq!(
        host.relieve_memory(&nothing_visible, now, Pressure::Warning),
        None
    );
    assert_eq!(
        host.relieve_memory(&nothing_visible, now, Pressure::Normal),
        None
    );
    // Twenty minutes later the recent one has been quiet for half an hour.
    let later = now + 20 * 60;
    assert_eq!(
        host.relieve_memory(&nothing_visible, later, Pressure::Warning),
        Some(ids[0].clone())
    );
    // The one that waits was never the valve's, even at critical...
    assert_eq!(
        host.relieve_memory(&nothing_visible, later + 120, Pressure::Critical),
        None
    );
    // ...and the hours do take it.
    assert_eq!(
        host.hibernate_what_is_due(&nothing_visible, now, 48),
        [ids[1].clone()]
    );
    assert_eq!(world.stored(&ids[1]).state, State::Hibernated);
}

// Audit of block 4 (7): a session looked at while it is still leaving after
// a hibernation opens by itself once its last run is gone, resuming.
#[test]
fn a_session_asked_for_during_the_grace_of_its_hibernation_wakes_by_itself() {
    let world = HostedWorld::new();
    let session = world.add_session("dormilona", Profile::Claude);
    world.steer_fake_claude("fake-claude.sleep", "30");
    world.steer_fake_claude("fake-claude.ignore-hup", "");
    let (mut host, _changes) = world.start();
    let id = run(&mut host, &world, "dormilona");
    world.wait_for_run(&id, 1);
    write_transcript(&world, &session.claude_session_id.unwrap());

    assert!(host.hibernate(&id));
    assert!(!host.reopens_when_closed(&id));
    let asked = host.open(
        &crate::hosted_world::target("dormilona"),
        crate::support::SIZE,
    );
    assert!(matches!(
        asked,
        Ok(lithemux_app::session_host::Opened::WhenClosed { .. })
    ));
    assert!(host.reopens_when_closed(&id));

    attend_until(&mut host, "the session to run again", |host| {
        host.is_running(&id)
    });
    let (_, flag) = world.wait_for_run(&id, 2);
    assert_eq!(flag, "--resume");
    assert!(!host.reopens_when_closed(&id));
}
