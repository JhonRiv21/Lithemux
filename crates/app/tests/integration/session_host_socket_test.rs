use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::time::{Duration, Instant};

use lithemux_app::session_host::{HostError, SessionHost};
use lithemux_core::modules::category::{FsBoardStore, SqliteCategoryRepository};
use lithemux_core::modules::context::ContextPorts;
use lithemux_core::modules::hook_event::{
    FileHookLog, HookPorts, SocketAppNotifier, handle_hook_event,
};
use lithemux_core::modules::session::{Profile, SqliteSessionRepository, State};
use lithemux_core::modules::settings::SqliteSettingsRepository;

use crate::hosted_world::{HostedWorld, attend_until, run};

/// What the hook binary does when Claude Code fires an event: write the
/// state and tell the app.
fn fire_hook(world: &HostedWorld, session_id: &str, event: &str, stdin: &str) {
    let db = world.db();
    let ports = HookPorts {
        context: ContextPorts {
            categories: &SqliteCategoryRepository::new(&db),
            sessions: &SqliteSessionRepository::new(&db),
            settings: &SqliteSettingsRepository::new(&db),
            boards: &FsBoardStore::new(&world.home),
        },
        log: &FileHookLog::new(&world.home),
        notifier: &SocketAppNotifier::new(&world.home),
    };
    handle_hook_event(&ports, session_id, event, stdin, 1_800_000_100);
}

// state in memory in under 100 ms
#[test]
fn a_hook_event_changes_the_state_in_memory_within_100_ms() {
    let world = HostedWorld::new();
    world.add_session("ejecutora", Profile::Claude);
    world.steer_fake_claude("fake-claude.sleep", "30");
    let (mut host, changes) = world.start();
    let id = run(&mut host, &world, "ejecutora");
    assert_eq!(host.state(&id), Some(State::New));

    // Whatever the start of the session left to attend is attended first.
    host.process_pending();
    while changes.try_recv().is_ok() {}

    let fired = Instant::now();
    fire_hook(&world, &id, "prompt-submit", "{}");
    // The app waits on this channel and then reads its mail. Here the wait
    // has a deadline, so a lost event fails the test instead of hanging it.
    let deadline = fired + Duration::from_secs(5);
    while host.state(&id) != Some(State::Working) {
        assert!(Instant::now() < deadline, "the event never arrived");
        if changes.try_recv().is_ok() {
            host.process_pending();
        }
    }
    let took = fired.elapsed();

    // Shown with `--nocapture`, for the phase report.
    println!("hook to memory: {took:?}");
    assert_eq!(host.state(&id), Some(State::Working));
    assert!(took < Duration::from_millis(100), "took {took:?}");
}

// PLAN §2.1: the state does not travel in the message; the database does
#[test]
fn the_state_comes_from_the_database_and_not_from_the_message() {
    let world = HostedWorld::new();
    let session = world.add_session("ejecutora", Profile::Claude);
    let (mut host, _changes) = world.start();
    fire_hook(&world, &session.id, "prompt-submit", "{}");
    attend_until(&mut host, "the first event", |host| {
        host.state(&session.id) == Some(State::Working)
    });

    // A message that claims an event the database never saw changes nothing.
    let mut stream = UnixStream::connect(world.home.app_socket_path()).unwrap();
    let claim = format!("{{\"session_id\":\"{}\",\"event\":\"stop\"}}\n", session.id);
    stream.write_all(claim.as_bytes()).unwrap();
    drop(stream);
    std::thread::sleep(Duration::from_millis(100));
    host.process_pending();

    assert_eq!(host.state(&session.id), Some(State::Working));
}

// PLAN §2.1: malformed messages and unknown sessions are ignored
#[test]
fn garbage_on_the_socket_does_not_stop_the_host_from_listening() {
    let world = HostedWorld::new();
    let session = world.add_session("ejecutora", Profile::Claude);
    let (mut host, _changes) = world.start();
    let socket = world.home.app_socket_path();

    for garbage in [
        &b"not json at all\n"[..],
        b"{\"session_id\":\"no-such-session\",\"event\":\"stop\"}\n",
        b"{\"session_id\":7}\n",
        b"\xff\xfe\x00\n",
        b"",
    ] {
        let mut stream = UnixStream::connect(&socket).unwrap();
        stream.write_all(garbage).unwrap();
    }
    // One that connects and never says a word.
    let _silent = UnixStream::connect(&socket).unwrap();

    fire_hook(&world, &session.id, "prompt-submit", "{}");
    attend_until(&mut host, "the event after the garbage", |host| {
        host.state(&session.id) == Some(State::Working)
    });
    assert_eq!(host.state("no-such-session"), None);
}

// PLAN §2.1: the socket is for its user alone
#[test]
fn the_socket_can_only_be_used_by_its_owner() {
    let world = HostedWorld::new();
    let (_host, _changes) = world.start();

    let mode = fs::metadata(world.home.app_socket_path())
        .unwrap()
        .permissions()
        .mode();

    assert_eq!(mode & 0o777, 0o600);
}

// one of a host that closed is gone
#[test]
fn a_socket_nobody_answers_on_is_replaced() {
    let world = HostedWorld::new();
    fs::create_dir_all(world.home.path()).unwrap();
    // Bound and dropped without removing the file: what a crash leaves.
    drop(UnixListener::bind(world.home.app_socket_path()).unwrap());
    assert!(world.home.app_socket_path().exists());

    let (host, _changes) = world.start();
    assert!(UnixStream::connect(world.home.app_socket_path()).is_ok());

    drop(host);
    assert!(!world.home.app_socket_path().exists());
}

// one byte at a time, do not hold up the events of the others
#[test]
fn silent_and_slow_clients_do_not_delay_an_event() {
    let world = HostedWorld::new();
    let session = world.add_session("ejecutora", Profile::Claude);
    let (mut host, _changes) = world.start();
    let socket = world.home.app_socket_path();
    let silent: Vec<UnixStream> = (0..5)
        .map(|_| UnixStream::connect(&socket).unwrap())
        .collect();
    let mut slow = UnixStream::connect(&socket).unwrap();
    let dripping = std::thread::spawn(move || {
        for byte in b"{\"session_id\":\"x\",\"ev" {
            if slow.write_all(&[*byte]).is_err() {
                return;
            }
            std::thread::sleep(Duration::from_millis(150));
        }
    });

    let fired = Instant::now();
    fire_hook(&world, &session.id, "prompt-submit", "{}");
    attend_until(&mut host, "the event behind the slow clients", |host| {
        host.state(&session.id) == Some(State::Working)
    });
    let took = fired.elapsed();

    assert!(took < Duration::from_millis(150), "took {took:?}");
    drop(silent);
    dripping.join().unwrap();
}

// removed, and the app does not start over it
#[test]
fn a_socket_somebody_answers_on_is_left_alone() {
    let world = HostedWorld::new();
    fs::create_dir_all(world.home.path()).unwrap();
    let other_program = UnixListener::bind(world.home.app_socket_path()).unwrap();

    let refused = SessionHost::start(world.config());

    assert!(matches!(refused.err(), Some(HostError::SocketInUse(_))));
    assert!(world.home.app_socket_path().exists());
    assert!(UnixStream::connect(world.home.app_socket_path()).is_ok());
    drop(other_program);
}

// Audit of block 3 (2): with a home whose path does not fit in a socket
// address the host still starts, and the hook still reaches it.
#[test]
fn a_home_too_long_for_a_socket_still_hears_the_hooks() {
    let world = HostedWorld::with_a_long_home();
    assert!(world.home.path().join("app.sock").as_os_str().len() > 104);
    world.add_session("ejecutora", Profile::Claude);
    world.steer_fake_claude("fake-claude.sleep", "30");
    let (mut host, _changes) = world.start();
    let id = run(&mut host, &world, "ejecutora");

    let socket = world.home.app_socket_path();
    assert!(!socket.starts_with(world.home.path()));
    assert!(socket.exists());
    fire_hook(&world, &id, "prompt-submit", "{}");
    attend_until(&mut host, "the hook to be heard", |host| {
        host.state(&id) == Some(State::Working)
    });

    drop(host);
    assert!(!socket.exists());
}
