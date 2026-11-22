//! PLAN §3.2 and §3.6 on the host: the version of Claude Code and the
//! app's log.

use std::fs;

use lithemux_app::session_host::Phase;
use lithemux_core::modules::launch::{FlockSessionLocks, SessionLocks};
use lithemux_core::modules::session::{Profile, State};

use crate::hosted_world::{HostedWorld, attend_until, open, run};
use crate::processes::wait_until_gone;

// PLAN §3.2: the version a launch would use now is asked in the background,
// the way a launch finds `claude`.
#[test]
fn the_installed_claude_version_is_the_one_a_launch_would_use() {
    let world = HostedWorld::new();
    let (host, _changes) = world.start();
    let ask = host.version_asker();
    let asked = std::thread::spawn(ask).join().unwrap();
    // The fake claude answers `--version` with this.
    assert_eq!(asked.as_deref(), Some("9.9.9"));
}

// PLAN §3.2: «reanudar las sesiones idle»: through hibernate and wake, with
// --resume, keeping the conversation; and only those asked for.
#[test]
fn restarting_a_session_resumes_its_conversation_on_the_installed_version() {
    let world = HostedWorld::new();
    let session = world.add_session("vieja", Profile::Claude);
    world.add_session("otra", Profile::Claude);
    world.steer_fake_claude("fake-claude.sleep", "30");
    world.steer_fake_claude("fake-claude.ignore-hup", "");
    let (mut host, _changes) = world.start();
    let id = run(&mut host, &world, "vieja");
    let other = run(&mut host, &world, "otra");
    let (first_pid, _) = world.wait_for_run(&id, 1);
    world.write_transcript(&session.claude_session_id.unwrap());

    assert!(host.restart(&id));
    attend_until(&mut host, "the session to run again", |host| {
        host.is_running(&id)
    });
    let (second_pid, flag) = world.wait_for_run(&id, 2);
    assert_ne!(first_pid, second_pid);
    assert_eq!(flag, "--resume");
    wait_until_gone(first_pid);
    assert!(host.is_running(&other));
    assert_eq!(
        world
            .fake_claude_runs()
            .iter()
            .filter(|(s, _, _)| *s == other)
            .count(),
        1
    );
    // What is not running is not restarted.
    assert!(!host.restart("no-such-session"));
}

// PLAN §3.2 «en lote y solo con las idle», and B4: among the sessions on
// an older version, the one at work and the one kept alive (a development
// server) are left running; only the idle one is resumed.
#[test]
fn the_batch_resumes_only_the_idle_sessions_that_are_not_kept_alive() {
    let world = HostedWorld::new();
    world.steer_fake_claude("fake-claude.sleep", "30");
    let (mut host, _changes) = world.start();
    let mut ids = Vec::new();
    for slug in ["reposo", "ocupada", "servidor", "al-dia"] {
        world.add_session(slug, Profile::Claude);
        ids.push(run(&mut host, &world, slug));
    }
    let [idle, busy, server, current] = [&ids[0], &ids[1], &ids[2], &ids[3]];
    for id in &ids {
        world.wait_for_run(id, 1);
        world.set_state(id, State::Idle);
    }
    world.set_state(busy, State::Working);
    world.keep_alive(server);
    for id in [idle, busy, server] {
        world.set_claude_version(id, "9.9.8");
    }

    let update = host.claude_update("9.9.9");
    assert_eq!(update.idle, std::slice::from_ref(idle));
    assert_eq!(update.busy, std::slice::from_ref(busy));

    assert_eq!(host.resume_idle_on("9.9.9"), std::slice::from_ref(idle));
    attend_until(&mut host, "the idle one to run again", |host| {
        host.is_running(idle)
    });
    world.wait_for_run(idle, 2);
    for untouched in [busy, server, current] {
        assert!(host.is_running(untouched));
        let runs = world.fake_claude_runs();
        assert_eq!(
            runs.iter().filter(|(s, _, _)| s == untouched).count(),
            1,
            "{untouched}"
        );
    }
}

fn app_log(world: &HostedWorld) -> String {
    fs::read_to_string(world.home.app_log_path()).unwrap_or_default()
}

// PLAN §3.6: the significant errors of the host go to <home>/logs/app.log,
// with level and time, and nothing of any conversation.
#[test]
fn a_session_that_cannot_start_and_one_open_elsewhere_are_logged() {
    let world = HostedWorld::new();
    let gone = world.add_session("sin-carpeta", Profile::Claude);
    let elsewhere = world.add_session("en-otra-terminal", Profile::Claude);
    let (mut host, _changes) = world.start();

    world.set_cwd(&gone.id, "/no/such/folder");
    let locks = FlockSessionLocks::new(&world.home);
    let _held = locks.acquire(&elsewhere.id).unwrap();
    assert!(matches!(open(&mut host, "sin-carpeta"), Phase::Failed(_)));
    assert!(matches!(
        open(&mut host, "en-otra-terminal"),
        Phase::OpenElsewhere { .. }
    ));

    let log = app_log(&world);
    let line = |needle: &str| {
        log.lines()
            .find(|line| line.contains(needle))
            .map(str::to_string)
    };
    let failed = line(&gone.id).unwrap();
    assert!(failed.contains(" ERROR could not start "), "{failed}");
    assert!(failed.contains("/no/such/folder"), "{failed}");
    let held = line(&elsewhere.id).unwrap();
    assert!(
        held.contains(" WARN ") && held.contains("open elsewhere"),
        "{held}"
    );
    // Every line starts with its time.
    for line in log.lines() {
        assert!(line.len() > 20 && line.as_bytes()[19] == b'Z', "{line}");
    }
}

// PLAN §3.6: a database the app makes, or migrates, says so in the log.
#[test]
fn a_database_made_by_the_app_is_logged() {
    let world = HostedWorld::new();
    for name in ["lithemux.db", "lithemux.db-wal", "lithemux.db-shm"] {
        let _ = fs::remove_file(world.home.path().join(name));
    }
    let (host, _changes) = world.start();
    assert!(app_log(&world).contains(" INFO database created at schema v"));
    drop(host);
    // Opened again, there is nothing new to say.
    let (host, _changes) = world.start();
    assert_eq!(app_log(&world).matches("database").count(), 1);
    drop(host);
}
