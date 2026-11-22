use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use crate::platform::paths::app_socket::{MAX_SOCKET_PATH_BYTES, app_socket};

fn long_home(tag: &str) -> PathBuf {
    Path::new("/Users/someone-with-a-rather-long-name/Library/Application Support")
        .join("a-folder-that-makes-the-path-too-long-for-a-socket-address")
        .join(tag)
}

// PLAN §2.1: the app listens on a Unix socket in its home.
#[test]
fn the_socket_is_in_the_home_when_its_path_fits() {
    let socket = app_socket(
        Path::new("/Users/ana/Library/Application Support/lithemux"),
        501,
    );
    assert_eq!(
        socket.path(),
        Path::new("/Users/ana/Library/Application Support/lithemux/app.sock")
    );
    assert!(!socket.is_outside_the_home());
    assert!(socket.is_private_to(501));
}

// Audit of block 3 (2): a path longer than a socket address holds must not
// keep the app from starting.
#[test]
fn a_home_too_long_for_a_socket_address_gets_a_short_private_path() {
    let home = long_home("one");
    assert!(home.join("app.sock").as_os_str().len() > MAX_SOCKET_PATH_BYTES);
    let socket = app_socket(&home, 501);
    assert!(socket.is_outside_the_home());
    assert!(socket.path().starts_with("/tmp/lithemux-501"));
    assert!(socket.path().as_os_str().len() <= MAX_SOCKET_PATH_BYTES);
}

// Audit of block 3 (2): the app and the hook compute it separately.
#[test]
fn the_same_home_always_gives_the_same_path_and_another_home_another() {
    assert_eq!(
        app_socket(&long_home("one"), 501),
        app_socket(&long_home("one"), 501)
    );
    assert_ne!(
        app_socket(&long_home("one"), 501).path(),
        app_socket(&long_home("two"), 501).path()
    );
    assert_ne!(
        app_socket(&long_home("one"), 501).path(),
        app_socket(&long_home("one"), 502).path()
    );
}

// Audit of block 3 (2)
#[test]
fn the_longest_path_that_fits_stays_in_the_home() {
    let fits = "/".to_string() + &"h".repeat(MAX_SOCKET_PATH_BYTES - "/app.sock".len() - 1);
    assert!(!app_socket(Path::new(&fits), 501).is_outside_the_home());
    assert!(app_socket(Path::new(&(fits + "h")), 501).is_outside_the_home());
}

// Audit of block 3 (2): `/tmp` is shared; the directory must be ours alone.
#[test]
fn the_short_directory_is_made_private_and_one_that_is_not_is_refused() {
    let ours = rustix::process::getuid().as_raw();
    // A user id of its own keeps this test away from a real app's directory.
    let made_up = 3_000_000_000 + std::process::id();
    let socket = app_socket(&long_home("private"), made_up);
    let dir = socket.path().parent().unwrap().to_path_buf();
    let _ = std::fs::remove_dir_all(&dir);

    // It does not exist: nothing to trust yet, and preparing it for a user
    // that is not its owner fails.
    assert!(!socket.is_private_to(ours));
    assert!(socket.prepare(made_up).is_err());
    // Made by us, private: good for us and for nobody else.
    assert!(socket.prepare(ours).is_ok());
    assert!(socket.is_private_to(ours));
    assert!(!socket.is_private_to(made_up));
    let mode = std::fs::metadata(&dir).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o700);

    // Opened to others: no longer trusted.
    std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert!(!socket.is_private_to(ours));
    assert!(socket.prepare(ours).is_err());

    // A symlink in its place is not followed.
    std::fs::remove_dir_all(&dir).unwrap();
    let target = tempfile::tempdir().unwrap();
    std::os::unix::fs::symlink(target.path(), &dir).unwrap();
    assert!(!socket.is_private_to(ours));
    assert!(socket.prepare(ours).is_err());
    std::fs::remove_file(&dir).unwrap();
}
