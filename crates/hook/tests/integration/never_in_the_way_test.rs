use std::os::unix::net::{UnixListener, UnixStream};
use std::time::{Duration, Instant};

use lithemux_core::modules::session::State;

use crate::sandbox::hook;
use crate::seeded_home::{SeededHome, fixture};

fn assert_silent_success(output: &std::process::Output) {
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stdout.is_empty(), "stdout: {:?}", output.stdout);
    assert!(output.stderr.is_empty(), "stderr: {:?}", output.stderr);
}

// PLAN §0.4 / §0.7: without LITHEMUX_SESSION_ID it does nothing and exits 0
#[test]
fn without_a_lithemux_session_it_exits_zero_with_no_output() {
    let seeded = SeededHome::new();
    for subcommand in [
        "session-start",
        "prompt-submit",
        "stop",
        "notification",
        "session-end",
    ] {
        let output = hook()
            .arg(subcommand)
            .env("LITHEMUX_HOME", seeded.dir.path())
            .write_stdin(fixture("session_start_startup.json"))
            .output()
            .unwrap();
        assert_silent_success(&output);
    }
    assert_eq!(
        seeded.executor_now().state,
        State::New,
        "nothing was touched"
    );
    assert!(!seeded.home.hook_log_path().exists());
}

// PLAN §0.4: an empty LITHEMUX_SESSION_ID is the same as none
#[test]
fn an_empty_session_id_counts_as_no_session() {
    let seeded = SeededHome::new();
    let output = hook()
        .arg("stop")
        .env("LITHEMUX_HOME", seeded.dir.path())
        .env("LITHEMUX_SESSION_ID", "")
        .output()
        .unwrap();
    assert_silent_success(&output);
}

// PLAN §0.4 / §0.7: with the database locked it still exits 0
#[test]
fn with_the_database_locked_it_exits_zero_and_writes_nothing_to_stderr() {
    let seeded = SeededHome::new();
    let blocker = seeded.db();
    blocker.conn().execute_batch("BEGIN EXCLUSIVE").unwrap();

    let started = Instant::now();
    let output = seeded
        .hook("stop")
        .write_stdin(fixture("stop.json"))
        .output()
        .unwrap();
    let elapsed = started.elapsed();
    blocker.conn().execute_batch("ROLLBACK").unwrap();

    assert_silent_success(&output);
    // busy_timeout is 250 ms; the Stop hook is allowed 2 s by the settings JSON.
    assert!(elapsed < Duration::from_secs(2), "took {elapsed:?}");
    let log = std::fs::read_to_string(seeded.home.hook_log_path()).unwrap();
    assert!(log.contains("stop"), "the failure is in the log: {log}");
    assert_eq!(seeded.executor_now().state, State::New);
}

// PLAN §0.4: a locked database must not cost a starting session its context
#[test]
fn with_the_database_locked_session_start_still_prints_the_context() {
    let seeded = SeededHome::new();
    let blocker = seeded.db();
    blocker.conn().execute_batch("BEGIN IMMEDIATE").unwrap();
    let output = seeded
        .hook("session-start")
        .write_stdin(fixture("session_start_startup.json"))
        .output()
        .unwrap();
    blocker.conn().execute_batch("ROLLBACK").unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("demo-ejecutora"), "{stdout}");
}

// PLAN §0.4: always exit 0, whatever happens
#[test]
fn nothing_makes_it_fail() {
    let seeded = SeededHome::new();
    let empty = tempfile::tempdir().unwrap();
    let cases: Vec<assert_cmd::Command> = vec![
        // no database at all
        {
            let mut c = hook();
            c.arg("stop")
                .env("LITHEMUX_HOME", empty.path())
                .env("LITHEMUX_SESSION_ID", "x");
            c
        },
        // a home that cannot exist
        {
            let mut c = hook();
            c.arg("stop")
                .env("LITHEMUX_HOME", "/dev/null/nope")
                .env("LITHEMUX_SESSION_ID", "x");
            c
        },
        // unknown event, no event, unknown session
        seeded.hook("pre-tool-use"),
        {
            let mut c = hook();
            c.env("LITHEMUX_HOME", seeded.dir.path())
                .env("LITHEMUX_SESSION_ID", &seeded.executor.id);
            c
        },
        {
            let mut c = hook();
            c.arg("session-start")
                .env("LITHEMUX_HOME", seeded.dir.path())
                .env("LITHEMUX_SESSION_ID", "not-a-known-session");
            c
        },
    ];
    for mut command in cases {
        let output = command.write_stdin("<<garbage>>").output().unwrap();
        assert_silent_success(&output);
    }
    // a corrupt database
    std::fs::write(empty.path().join("lithemux.db"), b"this is not sqlite").unwrap();
    let output = hook()
        .arg("stop")
        .env("LITHEMUX_HOME", empty.path())
        .env("LITHEMUX_SESSION_ID", "x")
        .output()
        .unwrap();
    assert_silent_success(&output);
}

const SOCKET_BUDGET: Duration = Duration::from_millis(500);

fn stop_takes(seeded: &SeededHome) -> Duration {
    let started = Instant::now();
    let output = seeded
        .hook("stop")
        .write_stdin(fixture("stop.json"))
        .output()
        .unwrap();
    assert_silent_success(&output);
    assert_eq!(seeded.executor_now().state, State::Idle);
    started.elapsed()
}

// PLAN §2.0 bug 10: an app that accepts and never reads costs the hook nothing
#[test]
fn an_app_that_accepts_but_never_reads_does_not_delay_the_hook() {
    let seeded = SeededHome::new();
    let listener = UnixListener::bind(seeded.home.app_socket_path()).unwrap();
    let accepting = std::thread::spawn(move || listener.accept().map(|(stream, _)| stream));

    let elapsed = stop_takes(&seeded);

    assert!(elapsed < SOCKET_BUDGET, "took {elapsed:?}");
    drop(accepting.join().unwrap());
}

// PLAN §2.0 bug 10: neither does an app too busy to accept at all
#[test]
fn an_app_that_never_accepts_does_not_delay_the_hook() {
    let seeded = SeededHome::new();
    let path = seeded.home.app_socket_path();
    let _listener = UnixListener::bind(&path).unwrap();
    // Fill the queue of pending connections, as a stuck app would have it.
    let pending: Vec<UnixStream> = (0..512)
        .map_while(|_| {
            let stream = UnixStream::connect(&path).ok()?;
            stream.set_nonblocking(true).ok()?;
            Some(stream)
        })
        .collect();

    let elapsed = stop_takes(&seeded);

    assert!(elapsed < SOCKET_BUDGET, "took {elapsed:?}");
    drop(pending);
}
