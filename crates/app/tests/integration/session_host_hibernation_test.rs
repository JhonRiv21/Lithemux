//! PLAN §2.4: hibernation, by the hours, by the memory valve and by hand,
//! with the grace a program gets to leave.

use lithemux_core::modules::launch::{FlockSessionLocks, SessionLocks};
use lithemux_core::modules::session::{Pressure, Profile, State};

use crate::hosted_world::{HostedWorld, attend_until, run};
use crate::processes::{is_alive, wait_until_gone};

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
    world.write_transcript(&session.claude_session_id.unwrap());

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
    world.write_transcript(&session.claude_session_id.unwrap());

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
