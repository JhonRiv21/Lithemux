use std::fs;

use lithemux_core::modules::hook_event::{FileHookLog, HookLog};

use crate::temp_home::TempHome;

// PLAN §0.4: errors go to <home>/logs/hook.log
#[test]
fn lines_are_appended_to_the_hook_log() {
    let temp = TempHome::new();
    let log = FileHookLog::new(&temp.home);
    log.append("first problem");
    log.append("second problem");
    let content = fs::read_to_string(temp.home.path().join("logs/hook.log")).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].ends_with("first problem"));
    assert!(lines[1].ends_with("second problem"));
}

// PLAN §0.4: rotated at 1 MB
#[test]
fn the_hook_log_is_rotated_at_one_megabyte() {
    let temp = TempHome::new();
    let logs = temp.home.path().join("logs");
    fs::create_dir_all(&logs).unwrap();
    fs::write(logs.join("hook.log"), vec![b'x'; 1024 * 1024]).unwrap();

    FileHookLog::new(&temp.home).append("after rotation");

    let current = fs::read_to_string(logs.join("hook.log")).unwrap();
    assert!(current.len() < 200);
    assert!(current.contains("after rotation"));
    assert_eq!(
        fs::metadata(logs.join("hook.log.1")).unwrap().len(),
        1024 * 1024
    );
}
