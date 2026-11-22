use std::time::{Duration, Instant};

use crate::sandbox::Sandbox;
use crate::signals;

const LIMIT: Duration = Duration::from_secs(7);

// PLAN §2.0 bug 1: a profile that leaves a background process holding stdout
// must not hang the lookup of `claude`.
#[test]
fn open_does_not_wait_for_what_the_shell_profile_left_running() {
    let sandbox = Sandbox::new()
        .with_fake_shell("fake-shell-lingering")
        .with_demo();
    let started = Instant::now();

    let output = sandbox
        .lithemux(&["open", "demo/ejecutora"])
        .output()
        .unwrap();

    assert!(started.elapsed() < LIMIT, "took {:?}", started.elapsed());
    if Sandbox::fakes_on_the_path_are_launchable() {
        assert!(output.status.success());
        assert!(sandbox.home().join("fake-claude.log").exists());
    }
}

// PLAN §2.0 bug 1
#[test]
fn doctor_does_not_wait_for_what_the_shell_profile_left_running() {
    let sandbox = Sandbox::new()
        .with_fake_shell("fake-shell-lingering")
        .with_demo();
    let started = Instant::now();

    let report = sandbox.lithemux(&["doctor"]).output().unwrap();

    assert!(started.elapsed() < LIMIT, "took {:?}", started.elapsed());
    let report = String::from_utf8_lossy(&report.stdout).to_string();
    // The fake found through the PATH of the fake shell.
    assert!(report.contains("fakes/on-path/"), "{report}");
}

// PLAN §2.0 bug 1: "matar el grupo entero": nothing the lookup started is
// left running
#[test]
fn what_the_shell_profile_left_running_is_gone_after_the_lookup() {
    let sandbox = Sandbox::new()
        .with_fake_shell("fake-shell-lingering")
        .with_demo();

    sandbox.lithemux(&["doctor"]).output().unwrap();

    let left_behind = std::fs::read_to_string(sandbox.home().join("left-behind.pids")).unwrap();
    assert!(left_behind.lines().count() >= 2, "{left_behind}");
    for pid in left_behind.lines() {
        signals::wait_until_gone(pid.parse().unwrap());
    }
}

// docs/decisiones.md D-23 and D-28 (audit of the delta): a job of the
// profile that prints a path while the shell answers can win the race, so
// what it prints must never be taken for claude, nor remembered
#[test]
fn a_path_printed_by_a_job_of_the_profile_never_becomes_claude() {
    let sandbox = Sandbox::new()
        .with_fake_shell("fake-shell-late-writer")
        .with_demo();

    for round in 0..20 {
        let report = sandbox.lithemux(&["doctor"]).output().unwrap();
        let report = String::from_utf8_lossy(&report.stdout).to_string();
        assert!(report.contains("fakes/on-path/"), "round {round}: {report}");
        assert!(!report.contains("/bin/ls"), "round {round}: {report}");
    }
    let remembered = sandbox.run(&["config", "get", "last_claude_path"]);
    assert!(!remembered.contains("/bin/ls"), "{remembered}");
}

// Audit of the delta: what the shell prints once it has answered (an exit
// hook, a job reporting back) is not part of the answer, even when it looks
// exactly like one
#[test]
fn what_the_shell_prints_after_answering_is_not_the_answer() {
    let sandbox = Sandbox::new()
        .with_fake_shell("fake-shell-trailing-output")
        .with_demo();

    let report = sandbox.lithemux(&["doctor"]).output().unwrap();

    let report = String::from_utf8_lossy(&report.stdout).to_string();
    assert!(report.contains("fakes/on-path/"), "{report}");
    assert!(!report.contains("fakes/decoy/"), "{report}");
}

/// The lookups share about four seconds, however many of them there are.
const HUNG_PROFILE_LIMIT: Duration = Duration::from_secs(5);

// Audit M2: with a profile that hangs, every lookup used to wait its own
// three seconds, one after another
#[test]
fn a_profile_that_hangs_costs_open_one_shared_wait() {
    let sandbox = Sandbox::new()
        .with_fake_shell("fake-shell-hanging")
        .with_demo();
    let started = Instant::now();

    let error = sandbox.fail(&["open", "demo/ejecutora"]);

    assert!(
        started.elapsed() < HUNG_PROFILE_LIMIT,
        "took {:?}",
        started.elapsed()
    );
    assert!(error.contains("could not find"), "{error}");
}

// Audit M2
#[test]
fn a_profile_that_hangs_costs_doctor_one_shared_wait() {
    let sandbox = Sandbox::new()
        .with_fake_shell("fake-shell-hanging")
        .with_demo();
    let started = Instant::now();

    sandbox.lithemux(&["doctor"]).output().unwrap();

    assert!(
        started.elapsed() < HUNG_PROFILE_LIMIT,
        "took {:?}",
        started.elapsed()
    );
}
