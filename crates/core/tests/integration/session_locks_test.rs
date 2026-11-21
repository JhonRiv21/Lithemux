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
    held.record_pid(4321);

    let second = locks.acquire("abc");

    let Err(error) = second else {
        panic!("the second lock must fail");
    };
    assert!(matches!(
        error,
        LaunchError::AlreadyOpen { pid: Some(4321) }
    ));
    assert_eq!(error.to_string(), "the session is already open (pid 4321)");
    assert_eq!(locks.holder("abc"), Some(4321));
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
