use lithemux_core::modules::launch::{FlockSessionLocks, LaunchError, SessionLocks};

use crate::temp_home::TempHome;

// PLAN §0.3: flock on <home>/locks/<session_id>.lock
#[test]
fn the_lock_file_lives_under_locks_named_after_the_session() {
    let temp = TempHome::new();
    let _held = FlockSessionLocks::new(&temp.home).acquire("abc").unwrap();
    assert!(temp.home.path().join("locks/abc.lock").is_file());
}

// PLAN §0.3: if taken, opening fails with "the session is already open"
#[test]
fn a_held_session_cannot_be_locked_again_and_reports_the_pid() {
    let temp = TempHome::new();
    let locks = FlockSessionLocks::new(&temp.home);
    let mut held = locks.acquire("abc").unwrap();
    // A pid that is alive for the whole test: this process.
    let pid = std::process::id();
    held.record_pid(pid);

    let second = locks.acquire("abc");

    let Err(error) = second else {
        panic!("the second lock must fail");
    };
    assert!(matches!(error, LaunchError::AlreadyOpen { pid: Some(found) } if found == pid));
    assert_eq!(
        error.to_string(),
        format!("the session is already open (pid {pid})")
    );
    assert_eq!(locks.holder("abc"), Some(pid));
}

// PLAN §0.3: held while the process lives, then free again
#[test]
fn a_released_lock_can_be_taken_again() {
    let temp = TempHome::new();
    let locks = FlockSessionLocks::new(&temp.home);
    drop(locks.acquire("abc").unwrap());
    assert_eq!(locks.holder("abc"), None);
    assert!(locks.acquire("abc").is_ok());
}

// PLAN §0.3: one lock per session
#[test]
fn different_sessions_do_not_block_each_other() {
    let temp = TempHome::new();
    let locks = FlockSessionLocks::new(&temp.home);
    let _one = locks.acquire("one").unwrap();
    assert!(locks.acquire("two").is_ok());
    assert_eq!(locks.holder("never-opened"), None);
}

// PLAN §2.0 bug 6: the pid of an earlier holder is not shown for a new one
#[test]
fn a_new_holder_never_shows_the_pid_of_the_previous_one() {
    let temp = TempHome::new();
    let locks = FlockSessionLocks::new(&temp.home);
    let mut first = locks.acquire("abc").unwrap();
    first.record_pid(4321);
    drop(first);

    let _second = locks.acquire("abc").unwrap();

    let Err(LaunchError::AlreadyOpen { pid }) = locks.acquire("abc") else {
        panic!("the lock is held");
    };
    assert_ne!(pid, Some(4321));
}

// PLAN §2.0 bug 6: asking who holds a session never gets in the way of
// someone taking it
#[test]
fn asking_for_the_holder_never_blocks_an_acquire() {
    let temp = TempHome::new();
    let home = temp.home.clone();
    let asking = std::thread::spawn(move || {
        let locks = FlockSessionLocks::new(&home);
        for _ in 0..2_000 {
            let _ = locks.holder("abc");
        }
    });
    let locks = FlockSessionLocks::new(&temp.home);
    let mut refused = 0;
    while !asking.is_finished() {
        if locks.acquire("abc").is_err() {
            refused += 1;
        }
    }
    asking.join().unwrap();
    assert_eq!(refused, 0);
}

// PLAN §2.0 bug 6: a lock file left by a process that died is not a holder
#[test]
fn a_lock_file_left_by_a_dead_process_has_no_holder() {
    let temp = TempHome::new();
    let locks = FlockSessionLocks::new(&temp.home);
    drop(locks.acquire("abc").unwrap());
    // No process has this pid: it is above the system's maximum.
    std::fs::write(temp.home.lock_path("abc"), "2147483646\n").unwrap();

    assert_eq!(locks.holder("abc"), None);
    assert!(locks.acquire("abc").is_ok());
}

/// What a process that crashed leaves in the lock file: a pid, longer than
/// any pid that will be written over it, that belongs to nobody.
fn leave_the_pid_of_a_dead_holder(temp: &TempHome) {
    std::fs::create_dir_all(temp.home.locks_dir()).unwrap();
    std::fs::write(temp.home.lock_path("abc"), "2147483646\n").unwrap();
}

// PLAN §2.0 bug 6 (audit M4): from the moment the lock is taken, the holder
// shown is the process that took it, whatever a dead one left in the file
#[test]
fn right_after_taking_the_lock_the_holder_is_the_process_that_took_it() {
    let temp = TempHome::new();
    leave_the_pid_of_a_dead_holder(&temp);
    let locks = FlockSessionLocks::new(&temp.home);

    let _held = locks.acquire("abc").unwrap();

    let me = std::process::id();
    assert_eq!(locks.holder("abc"), Some(me));
    assert!(matches!(
        locks.acquire("abc"),
        Err(LaunchError::AlreadyOpen { pid: Some(pid) }) if pid == me
    ));
    assert_eq!(
        std::fs::read_to_string(temp.home.lock_path("abc")).unwrap(),
        format!("{me}\n")
    );
}

// PLAN §2.0 bug 6 (audit M4): and the pid of the session replaces it cleanly
#[test]
fn recording_the_session_pid_leaves_nothing_of_what_was_there() {
    let temp = TempHome::new();
    leave_the_pid_of_a_dead_holder(&temp);
    let locks = FlockSessionLocks::new(&temp.home);
    let mut held = locks.acquire("abc").unwrap();

    held.record_pid(7);

    assert_eq!(
        std::fs::read_to_string(temp.home.lock_path("abc")).unwrap(),
        "7\n"
    );
    drop(held);
    assert_eq!(
        std::fs::read_to_string(temp.home.lock_path("abc")).unwrap(),
        ""
    );
}
